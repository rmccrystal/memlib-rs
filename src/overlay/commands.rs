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
    Line(LineData),
    Box(BoxData),
    Text(TextData),
    Circle(CircleData),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LineData {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub color: u32,
    pub width: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BoxData {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub color: u32,
    pub rounding: f32,
    pub width: f32,
    pub filled: bool,
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
pub struct TextData {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub color: u32,
    pub font: Font,
    pub font_size: f32, // 0.0 for default font size
    pub centered: bool,
    pub style: TextStyle,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CircleData {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub color: u32,
    pub filled: bool,
    pub width: f32
}
