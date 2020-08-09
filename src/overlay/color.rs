/// An RGBA color
#[derive(Copy, Clone, Debug)]
pub struct Color(u32);

impl Color {
    /// Creates a color from an RGBA unsigned int
    pub fn new(color: u32) -> Self {
        Self(color)
    }

    pub fn as_int(&self) -> u32 {
        self.0
    }

    pub fn from_hsv(h: u8, s: u8, v: u8) -> Self {
        let mut h = h as f32 *   2.0; // 0-360
        let mut s = s as f32 / 255.0; // 0.0-1.0
        let mut v = v as f32 / 255.0; // 0.0-1.0

        let mut r = 0.0;
        let mut g = 0.0;
        let mut b = 0.0;

        let hi = (h / 60.0) as i32 % 6;
        let f  = (h / 60.0) - hi as f32;
        let p  = v * (1.0 - s);
        let q  = v * (1.0 - s * f);
        let t  = v * (1.0 - s * (1.0 - f));

        match hi {
            0 => { r = v; g = t; b = p; }
            1 => { r = q; g = v; b = p; }
            2 => { r = p; g = v; b = t; }
            3 => { r = p; g = q; b = v; }
            4 => { r = t; g = p; b = v; }
            5 => { r = v; g = p; b = q; }
            _ => {}
        }

        Self::from_rgb(r as u8 * 255, g as u8 * 255, b as u8 * 255)
    }

    pub fn from_hex(hex: u32) -> Self {
        Self::new(hex + 0xFF000000)
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
