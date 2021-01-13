use log::*;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use super::types::*;
use crate::math::Vector2;
use crate::overlay::OverlayInterface;
use std::collections::VecDeque;

pub struct LookingGlassOverlay {
    pipe: File,
    delay_buf: VecDeque<Command>,
    frame: Frame,
    delay: usize, // if the overlay is faster than the screen, there should be a delay
    anti_aliasing: bool,
}

impl LookingGlassOverlay {
    pub fn new(path: impl AsRef<Path>, anti_aliasing: bool, delay: usize) -> io::Result<Self> {
        let pipe = OpenOptions::new()
            .write(true)
            .append(true)
            .custom_flags(2048) // libc::O_NONBLOCK
            .open(path)?;

        Ok(Self {
            pipe,
            delay_buf: VecDeque::new(),
            frame: Frame::new(),
            delay,
            anti_aliasing,
        })
    }

    /// internal function
    fn _send_command(&mut self, command: Command) {
        trace!("Sending command to looking glass: {:?}", command);
        let buf = bincode::serialize(&command)
            .expect("Failed to serialize command for looking-glass overlay");
        if let Err(err) = self.pipe.write_all(buf.as_slice()) {
            error!("Error sending to looking-glass pipe: {}", err);
        }
    }

    /// Sends a command with delay
    fn send_command(&mut self, command: Command) {
        if self.delay == 0 {
            self._send_command(command);
            return;
        }

        // Add the buffer to the back of the delay_buf
        self.delay_buf.push_back(command);

        // If we should still add shit to the delay buffer
        if self.delay_buf.len() < self.delay {
            return;
        }

        let command = self
            .delay_buf
            .pop_front()
            .expect("Error with delay_buf on looking-glass overlay");
        self._send_command(command);
    }

    fn add_draw_command(&mut self, draw_command: DrawCommand) {
        self.frame.commands.push(draw_command);
    }
}

impl OverlayInterface for LookingGlassOverlay {
    fn begin(&mut self) {
        self.frame.commands.clear();
    }

    fn end(&mut self) {
        self.send_command(Command::UpdateFrame(self.frame.clone()));
    }

    fn draw_line(&mut self, p1: Vector2, p2: Vector2, options: LineOptions) {
        // if !self.anti_aliasing {
        //     p1 = p1.round();
        //     p2 = p2.round();
        // }
        self.add_draw_command(DrawCommand::Line {
            p1: p1.as_tuple(),
            p2: p2.as_tuple(),
            options,
        })
    }

    fn draw_box(&mut self, mut p1: Vector2, mut p2: Vector2, options: BoxOptions) {
        if !self.anti_aliasing {
            p1 = p1.round();
            p2 = p2.round();
        }
        self.add_draw_command(DrawCommand::Box {
            p1: p1.as_tuple(),
            p2: p2.as_tuple(),
            options,
        })
    }

    fn draw_text(&mut self, origin: Vector2, text: &str, options: TextOptions) {
        self.add_draw_command(DrawCommand::Text {
            origin: origin.as_tuple(),
            text: text.to_string(),
            options,
        })
    }

    fn draw_circle(&mut self, origin: Vector2, radius: f32, options: CircleOptions) {
        self.add_draw_command(DrawCommand::Circle {
            origin: origin.as_tuple(),
            radius,
            options,
        })
    }
}

impl Drop for LookingGlassOverlay {
    fn drop(&mut self) {
        self.end();
    }
}
