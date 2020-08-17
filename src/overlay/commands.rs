use serde::{Serialize, Deserialize};

/// Commands are sent through the pipe to control the overlay
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Command {
    UpdateFrame(Frame),
    ClearScreen
}

/// A frame state of the overlay
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Frame {
    pub commands: Vec<DrawCommand>
}

impl Frame {
    pub fn new() -> Self {
        Self{commands: vec![]}
    }
}

/// Represents data that can be drawn to the screen
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DrawCommand {
    Line {
        p1: Point,
        p2: Point,
        options: LineOptions
    },
    Box {
        p1: Point,
        p2: Point,
        options: BoxOptions,
    },
    Text {
        origin: Point,
        text: String,
        options: TextOptions,
    },
    Circle {
        origin: Point,
        radius: f32,
        options: CircleOptions,
    },
}

const DEFAULT_COLOR: u32 = 0xFFFFFFFF;

pub type Point = (f32, f32);

// Generate consuming setters with a macro
macro_rules! generate_setter {
    ($member_name:ident: $setter_type:ty) => {
        pub fn $member_name(mut self, $member_name: $setter_type) -> Self {
            self.$member_name = $member_name.into();
            self
        }
    };
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LineOptions {
    pub color: u32,
    pub width: f32,
}

impl Default for LineOptions {
    fn default() -> Self {
        LineOptions {
            color: DEFAULT_COLOR,
            width: 1.0
        }
    }
}

impl LineOptions {
    generate_setter!(color: impl Into<u32>);
    generate_setter!(width: f32);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BoxOptions {
    pub color: u32,
    pub rounding: f32,
    pub width: f32,
    pub filled: bool,
}

impl Default for BoxOptions {
    fn default() -> Self {
        Self {
            color: DEFAULT_COLOR,
            rounding: 0.0,
            width: 1.0,
            filled: false
        }
    }
}

impl BoxOptions {
    generate_setter!(color: impl Into<u32>);
    generate_setter!(rounding: f32);
    generate_setter!(width: f32);
    generate_setter!(filled: bool);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextOptions {
    pub color: u32,
    pub font: Font,
    pub font_size: Option<f32>, // if this is none, the font size will be default
    pub centered_horizontal: bool,
    pub centered_vertical: bool,
    pub style: TextStyle,
    pub shadow_color: u32,
}

impl Default for TextOptions {
    fn default() -> Self {
        Self {
            color: DEFAULT_COLOR,
            font: Font::Verdana,
            font_size: None,
            centered_horizontal: false,
            centered_vertical: false,
            style: TextStyle::Shadow,
            shadow_color: 0xAA000000
        }
    }
}

impl TextOptions {
    pub fn color(mut self, color: impl Into<u32>) -> Self {
        self.color = color.into();
        // set the opacity to the same opacity as color
        let opacity = ((self.color & 0xFF000000) >> 24) as u8;
        self.shadow_color = (self.shadow_color & 0x00FFFFFF) + ((opacity as u32) << 24);
        self
    }
    generate_setter!(font: Font);
    generate_setter!(font_size: Option<f32>);
    generate_setter!(centered_horizontal: bool);
    generate_setter!(centered_vertical: bool);
    generate_setter!(style: TextStyle);
    generate_setter!(shadow_color: impl Into<u32>);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TextStyle {
    None,
    Shadow,
    Outlined,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Font {
    Default,
    Pixel,
    Tahoma,
    Verdana
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CircleOptions {
    pub color: u32,
    pub filled: bool,
    pub width: f32
}

impl Default for CircleOptions {
    fn default() -> Self {
        Self {
            color: DEFAULT_COLOR,
            filled: false,
            width: 1.0
        }
    }
}

impl CircleOptions {
    generate_setter!(color: impl Into<u32>);
    generate_setter!(filled: bool);
    generate_setter!(width: f32);
}
