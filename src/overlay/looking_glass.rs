use log::*;
use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Write, Bytes};
use std::os::unix::fs::OpenOptionsExt;
use super::Color;
use crate::overlay::{OverlayInterface, TextStyle};
use std::collections::VecDeque;
use super::commands::*;
use crate::math::Vector2;

pub struct LookingGlassOverlay {
    pipe: File,
    delay_buf: VecDeque<Command>,
    frame: Frame,
    delay: usize,       // if the overlay is faster than the screen, there should be a delay
    anti_aliasing: bool
}

impl LookingGlassOverlay {
    pub fn new(path: impl AsRef<Path>, anti_aliasing: bool, delay: usize) -> io::Result<Self> {
        let pipe = OpenOptions::new()
            .write(true)
            .append(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)?;

        Ok(Self {
            pipe,
            delay_buf: VecDeque::new(),
            frame: Frame::new(),
            delay,
            anti_aliasing
        })
    }

    /// internal function
    fn _send_command(&mut self, command: Command) {
        trace!("Sending command to looking glass: {:?}", command);
        let buf = bincode::serialize(&command).expect("Failed to serialize command for looking-glass overlay");
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

        let command = self.delay_buf.pop_front().expect("Error with delay_buf on looking-glass overlay");
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

    fn draw_line(&mut self, mut p1: Vector2, mut p2: Vector2, color: Color, width: f32) {
        // if !self.anti_aliasing {
        //     p1 = p1.round();
        //     p2 = p2.round();
        // }
        self.add_draw_command(DrawCommand::Line(LineData {
            x1: p1.x,
            y1: p1.y,
            x2: p2.x,
            y2: p2.y,
            color: color.as_int(),
            width
        }))
    }

    fn draw_box(&mut self, mut p1: Vector2, mut p2: Vector2, color: Color, width: f32, rounding: f32, filled: bool) {
        if !self.anti_aliasing {
            p1 = p1.round();
            p2 = p2.round();
        }
        self.add_draw_command(DrawCommand::Box(BoxData {
            x1: p1.x,
            y1: p1.y,
            x2: p2.x,
            y2: p2.y,
            color: color.as_int(),
            rounding,
            width,
            filled
        }))
    }

    /// font_size = 0 for default size
    fn draw_text(&mut self, origin: Vector2, text: &str, color: Color, style: TextStyle, font: super::Font, font_size: f32, centered: bool) {
        self.add_draw_command(DrawCommand::Text(TextData {
            x: origin.x,
            y: origin.y,
            text: text.to_string(),
            color: color.as_int(),
            font,
            font_size,
            centered,
            style,
        }))
    }
}

impl Drop for LookingGlassOverlay {
    fn drop(&mut self) {
        self.end();
    }
}

#[repr(C)]
struct LgNull
{
    _type: u8,
    idx: u16,
}

#[repr(C)]
struct LgLine
{
    _type: u8,
    idx: u16,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    color: Color,
    width: f32,
}

#[repr(C)]
struct LgBox
{
    _type: u8,
    idx: u16,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    color: Color,
    thickness: f32,
    filled: bool,
}

#[repr(C)]
struct LgText
{
    _type: u8,
    idx: u16,
    x: f32,
    y: f32,
    size: u8,
    color: Color,
    style: TextStyle,
    str: [u8; 128],
}