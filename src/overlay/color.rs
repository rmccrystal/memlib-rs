/// An RGBA color
pub struct Color(u32);

impl Color {
    /// Creates a color from an RGBA unsigned int
    pub fn new(color: u32) -> Self {
        Self(color)
    }

    /// Creates a Color from 0-255 RGBA values.
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(((a as u32) << 24) | ((b as u32) << 16) | ((g as u32) << 8) | r as u32)
    }

    /// Creates a color from 0-255 RGB values. A is set to 255.
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba(r, g, b, 255)
    }
}
