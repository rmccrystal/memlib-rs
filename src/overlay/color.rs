/// An RGBA color
// stored as AGBR for some reason
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

    pub fn from_hex(mut hex: u32) -> Self {
        // If the hex doesn't include an A value, assume 0xFF
        if hex <= 0xFFFFFF {
            hex <<= 8;
            hex += 0xFF;
        }
        hex = hex.swap_bytes();
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
            ((self.0 >> 24) & 0xFF) as u8,
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

// Colors from https://blueprintjs.com/docs/#core/colors
impl Color {
    static_color!(black, 0x10161A);
    static_color!(blue1, 0x0E5A8A);
    static_color!(blue2, 0x106BA3);
    static_color!(blue3, 0x137CBD);
    static_color!(blue4, 0x2B95D6);
    static_color!(blue5, 0x48AFF0);
    static_color!(cobalt1, 0x1F4B99);
    static_color!(cobalt2, 0x2458B3);
    static_color!(cobalt3, 0x2965CC);
    static_color!(cobalt4, 0x4580E6);
    static_color!(cobalt5, 0x669EFF);
    static_color!(dark_gray1, 0x182026);
    static_color!(dark_gray2, 0x202B33);
    static_color!(dark_gray3, 0x293742);
    static_color!(dark_gray4, 0x30404D);
    static_color!(dark_gray5, 0x394B59);
    static_color!(forest1, 0x1D7324);
    static_color!(forest2, 0x238C2C);
    static_color!(forest3, 0x29A634);
    static_color!(forest4, 0x43BF4D);
    static_color!(forest5, 0x62D96B);
    static_color!(gold1, 0xA67908);
    static_color!(gold2, 0xBF8C0A);
    static_color!(gold3, 0xD99E0B);
    static_color!(gold4, 0xF2B824);
    static_color!(gold5, 0xFFC940);
    static_color!(gray1, 0x5C7080);
    static_color!(gray2, 0x738694);
    static_color!(gray3, 0x8A9BA8);
    static_color!(gray4, 0xA7B6C2);
    static_color!(gray5, 0xBFCCD6);
    static_color!(green1, 0x0A6640);
    static_color!(green2, 0x0D8050);
    static_color!(green3, 0x0F9960);
    static_color!(green4, 0x15B371);
    static_color!(green5, 0x3DCC91);
    static_color!(indigo1, 0x5642A6);
    static_color!(indigo2, 0x634DBF);
    static_color!(indigo3, 0x7157D9);
    static_color!(indigo4, 0x9179F2);
    static_color!(indigo5, 0xAD99FF);
    static_color!(light_gray1, 0xCED9E0);
    static_color!(light_gray2, 0xD8E1E8);
    static_color!(light_gray3, 0xE1E8ED);
    static_color!(light_gray4, 0xEBF1F5);
    static_color!(light_gray5, 0xF5F8FA);
    static_color!(lime1, 0x728C23);
    static_color!(lime2, 0x87A629);
    static_color!(lime3, 0x9BBF30);
    static_color!(lime4, 0xB6D94C);
    static_color!(lime5, 0xD1F26D);
    static_color!(orange1, 0xA66321);
    static_color!(orange2, 0xBF7326);
    static_color!(orange3, 0xD9822B);
    static_color!(orange4, 0xF29D49);
    static_color!(orange5, 0xFFB366);
    static_color!(red1, 0xA82A2A);
    static_color!(red2, 0xC23030);
    static_color!(red3, 0xDB3737);
    static_color!(red4, 0xF55656);
    static_color!(red5, 0xFF7373);
    static_color!(rose1, 0xA82255);
    static_color!(rose2, 0xC22762);
    static_color!(rose3, 0xDB2C6F);
    static_color!(rose4, 0xF5498B);
    static_color!(rose5, 0xFF66A1);
    static_color!(sepia1, 0x63411E);
    static_color!(sepia2, 0x7D5125);
    static_color!(sepia3, 0x96622D);
    static_color!(sepia4, 0xB07B46);
    static_color!(sepia5, 0xC99765);
    static_color!(turquoise1, 0x008075);
    static_color!(turquoise2, 0x00998C);
    static_color!(turquoise3, 0x00B3A4);
    static_color!(turquoise4, 0x14CCBD);
    static_color!(turquoise5, 0x2EE6D6);
    static_color!(vermilion1, 0x9E2B0E);
    static_color!(vermilion2, 0xB83211);
    static_color!(vermilion3, 0xD13913);
    static_color!(vermilion4, 0xEB532D);
    static_color!(vermilion5, 0xFF6E4A);
    static_color!(violet1, 0x5C255C);
    static_color!(violet2, 0x752F75);
    static_color!(violet3, 0x8F398F);
    static_color!(violet4, 0xA854A8);
    static_color!(violet5, 0xC274C2);
    static_color!(white, 0xFFFFFF);
}

#[test]
fn test_to_rgba() {
    // let color = Color::from_rgba(1, 2, 3, 4);
    // assert_eq!(color.to_rgba(), (1, 2, 3, 4));
    // assert_eq!(color.0, 0x_01_02_03_04)
}
