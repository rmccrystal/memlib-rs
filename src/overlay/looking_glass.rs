use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Write, Bytes};
use std::os::unix::fs::OpenOptionsExt;
use super::Color;
use crate::overlay::OverlayInterface;

pub struct LookingGlassOverlay {
    pipe: File,
    index: u16,
    largest_index: u16,
    buf: Vec<u8>,
}

impl LookingGlassOverlay {
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let pipe = OpenOptions::new()
            .write(true)
            .append(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)?;

        Ok(Self {
            pipe,
            index: 0,
            largest_index: 255,
            buf: Vec::new(),
        })
    }

    pub fn begin(&mut self) {
        self.buf.clear();
    }

    pub fn end(&mut self) {
        // self.index - 1 is last index
        for idx in self.index..self.largest_index {
            self.write_buf(to_bytes(&LgNull { _type: 0, idx }));
        }
        // Write the buffer
        self.pipe.write_all(self.buf.as_slice()).expect("Failed to write to looking-glass named pipe");

        // Update largest_index because we already flushed
        self.largest_index = self.index;
        self.index = 0;
    }

    fn write_buf(&mut self, buf: &[u8]) {
        self.buf.extend_from_slice(buf);
    }

    pub fn send_draw_command<T>(&mut self, command: &T) {
        let bytes = to_bytes(command);
        self.write_buf(&bytes);
        if self.index > self.largest_index {
            self.largest_index = self.index;
        }
        self.index += 1;
    }

}

impl OverlayInterface for LookingGlassOverlay {
    fn draw_line(&mut self, p1: (i32, i32), p2: (i32, i32), color: Color, width: i32) {
        let buf = LgLine {
            _type: 1,
            idx: self.index,
            x1: p1.0 as _,
            y1: p1.1 as _,
            x2: p2.0 as _,
            y2: p2.1 as _,
            color,
            width: width as _,
        };
        self.send_draw_command(&buf);
    }

    fn draw_box(&mut self, p1: (i32, i32), p2: (i32, i32), color: Color, thickness: i32) {
        let buf = LgBox {
            _type: 2,
            idx: self.index,
            x1: p1.0 as _,
            y1: p1.1 as _,
            x2: p2.0 as _,
            y2: p2.1 as _,
            color,
            thickness: thickness as _,
            filled: false,
        };
        self.send_draw_command(&buf)
    }

    fn draw_box_filled(&mut self, p1: (i32, i32), p2: (i32, i32), color: Color) {
        let buf = LgBox {
            _type: 2,
            idx: self.index,
            x1: p1.0 as _,
            y1: p1.1 as _,
            x2: p2.0 as _,
            y2: p2.1 as _,
            color,
            thickness: 0 as _,
            filled: true,
        };
        self.send_draw_command(&buf)
    }

    fn draw_text(&mut self, origin: (i32, i32), text: String, color: Color, size: u8) {
        let text_slice = text.as_bytes();

        // Create buffer
        let mut text_buf: [u8; 128] = [0; 128];
        for i in 0..128 {
            match text_slice.get(i) {
                Some(ch) => { text_buf[i] = *ch },
                None => break
            }
        }

        let buf = LgText {
            _type: 3,
            idx: self.index,
            x: origin.0 as _,
            y: origin.1 as _,
            size,
            color,
            str: text_buf,
        };

        self.send_draw_command(&buf)
    }
}

impl Drop for LookingGlassOverlay {
    fn drop(&mut self) {
        dbg!("drop");
        self.end();
    }
}

fn to_bytes<T>(ptr: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts(ptr as *const _ as *const u8, std::mem::size_of::<T>()) }
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
    str: [u8; 128],
}