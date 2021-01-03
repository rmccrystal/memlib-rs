/// An RGBA color
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color(u32);

impl Color {
    /// Creates a color from an RGBA unsigned int
    pub fn new(color: u32) -> Self {
        Self(color)
    }

    pub fn as_int(&self) -> u32 {
        self.0
    }

    pub fn from_hsv(h: f32, mut s: f32, mut v: f32) -> Self {
        s /= 100.0;
        v /= 100.0;

        let mut r = 0.0;
        let mut g = 0.0;
        let mut b = 0.0;

        let hi = (h / 60.0) as i32 % 6;
        let f = (h / 60.0) - hi as f32;
        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));

        match hi {
            0 => {
                r = v;
                g = t;
                b = p;
            }
            1 => {
                r = q;
                g = v;
                b = p;
            }
            2 => {
                r = p;
                g = v;
                b = t;
            }
            3 => {
                r = p;
                g = q;
                b = v;
            }
            4 => {
                r = t;
                g = p;
                b = v;
            }
            5 => {
                r = v;
                g = p;
                b = q;
            }
            _ => {}
        }

        Self::from_rgb((r * 255.0) as _, (g * 255.0) as _, (b * 255.0) as _)
    }

    pub fn from_hex(hex: u32) -> Self {
        let hex = hex.swap_bytes();
        Self::new(hex)
    }

    /// Creates a Color from 0-255 RGBA values.
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(((a as u32) << 24) | ((b as u32) << 16) | ((g as u32) << 8) | r as u32)
    }

    /// Creates a color from 0-255 RGB values. A is set to 255.
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba(r, g, b, 255)
    }

    pub fn to_rgba(&self) -> (u8, u8, u8, u8) {
        (
            (((self.0) & 0xFF) as u8),
            ((self.0 >> 8) & 0xFF) as u8,
            ((self.0 >> 16) & 0xFF) as u8,
            ((self.0 >> 24) & 0xFF) as u8
        )
    }

    pub fn opacity(&self, opacity: u8) -> Self {
        Self((self.0 & 0x00FFFFFF) + ((opacity as u32) << 24))
    }

    pub fn get_opacity(&self) -> u8 {
        ((self.0 & 0xFF000000) >> 24) as u8
    }
}

impl From<Color> for u32 {
    fn from(color: Color) -> Self {
        color.0
    }
}

impl From<u32> for Color {
    fn from(val: u32) -> Self {
        Color::new(val)
    }
}

macro_rules! static_color {
    ($name:ident,$hex:literal) => {
        pub fn $name() -> Self {
            Self::from_hex($hex)
        }
    };
}

impl Color {
    static_color!(red, 0xC0392B);
    static_color!(orange, 0xF39C12);
    static_color!(purple, 0x6C3483);
    static_color!(blue, 0x6C3483);
    static_color!(green, 0x6C3483);
    static_color!(yellow, 0xF4D03F);
    static_color!(white, 0xFFFFFF);
    static_color!(black, 0x000000);
    static_color!(light_gray, 0x839192);
    static_color!(gray, 0x839192);
    static_color!(dark_gray, 0x839192);
}

#[test]
fn test_to_rgba() {
    // let color = Color::from_rgba(1, 2, 3, 4);
    // assert_eq!(color.to_rgba(), (1, 2, 3, 4));
    // assert_eq!(color.0, 0x_01_02_03_04)
}
