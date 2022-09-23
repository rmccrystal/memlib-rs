#![allow(deprecated)]

/// Represents a type that can create and render a type that implements Draw for drawing on an overlay
pub trait Render {
    /// The frame type that the renderer generates
    type Frame: Draw;

    /// Adds a custom font by raw ttf data, the font size, and the id used to specify the
    /// font to use using Font::Custom. Panics if the font data cannot be parsed.
    fn add_custom_font(&mut self, font_data: Vec<u8>, font_size: f32, id: FontId) -> Option<()>;

    /// Gets the size of the frame
    fn frame_size(&self) -> (u32, u32);

    /// Creates a new instance of the Frame type
    fn frame(&mut self) -> &mut Self::Frame;

    /// Renders a modified frame crated by the frame function
    fn render(&mut self);
}

pub type Point = (f32, f32);
/// [R, G, B, A]
#[derive(Copy, Clone, Debug)]
pub struct Color([u8; 4]);

impl std::ops::Index<usize> for Color {
    type Output = u8;

    #[inline(always)]
    fn index(&self, index: usize) -> &u8 {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for Color {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.0[index]
    }
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color([r, g, b, a])
    }

    pub fn from_argb(argb: u32) -> Color {
        let b = argb as u8;
        let g = (argb >> 8) as u8;
        let r = (argb >> 16) as u8;
        let a = (argb >> 24) as u8;
        Color([r, g, b, a])
    }
}

pub type FontId = u32;

#[derive(Hash, PartialEq, Eq, Clone)]
pub enum Font {
    Default,
    Pixel,
    Custom(u32),
}

pub struct TextOptions {
    pub font: Font,
    pub font_size: f32,
}

pub struct LineOptions {
    pub thickness: f32,
}

pub struct RectOptions {
    pub thickness: f32,
    pub rounding: f32,
}

pub struct CircleOptions {
    pub thickness: f32,
}

/// Represents a type that can be directly drew on
pub trait Draw: Sized {
    #[deprecated(note = "Use the DrawExt trait instead of calling the Draw functions directly")]
    fn draw_text(&mut self, origin: Point, text: &str, color: Color, options: TextOptions);

    #[deprecated(note = "Use the DrawExt trait instead of calling the Draw functions directly")]
    fn draw_line(&mut self, p1: Point, p2: Point, color: Color, options: LineOptions);

    #[deprecated(note = "Use the DrawExt trait instead of calling the Draw functions directly")]
    fn draw_rect(&mut self, p1: Point, p2: Point, color: Color, options: RectOptions);

    #[deprecated(note = "Use the DrawExt trait instead of calling the Draw functions directly")]
    fn draw_rect_filled(&mut self, p1: Point, p2: Point, color: Color, options: RectOptions);

    #[deprecated(note = "Use the DrawExt trait instead of calling the Draw functions directly")]
    fn draw_circle(&mut self, origin: Point, radius: f32, color: Color, options: CircleOptions);

    #[deprecated(note = "Use the DrawExt trait instead of calling the Draw functions directly")]
    fn draw_circle_filled(&mut self, origin: Point, radius: f32, color: Color, options: CircleOptions);
}


/// Extension trait for Draw containing drawing builders for a cleaner drawing api.
/// These functions should be used instead of the raw Draw functions
pub trait DrawExt: Draw {
    fn text<'draw, 's>(&'draw mut self, origin: impl Into<Point>, text: &'s str, color: impl Into<Color>) -> Text<'draw, 's, Self> {
        Text { draw: self, origin: origin.into(), text, color: color.into(), font: Font::Default, font_size: 10., shadow_color: Color::from_argb(0xAA000000), style: TextStyle::Shadow }
    }

    fn line(&mut self, p1: impl Into<Point>, p2: impl Into<Point>, color: impl Into<Color>) -> Line<Self> {
        Line { draw: self, p1: p1.into(), p2: p2.into(), color: color.into(), thickness: 1. }
    }

    fn rect(&mut self, p1: impl Into<Point>, p2: impl Into<Point>, color: impl Into<Color>) -> Rect<Self> {
        Rect { draw: self, p1: p1.into(), p2: p2.into(), color: color.into(), thickness: 1., rounding: 1., filled: false }
    }

    fn circle(&mut self, origin: impl Into<Point>, radius: impl Into<f32>, color: impl Into<Color>) -> Circle<Self> {
        Circle { draw: self, origin: origin.into(), radius: radius.into(), color: color.into(), filled: false, thickness: 1. }
    }
}

impl<T: Draw> DrawExt for T {}

macro setter($member_name:ident, $setter_type:ty) {
pub fn $member_name(mut self, $member_name: impl Into<$setter_type>) -> Self {
    self.$member_name = $member_name.into();
    self
}
}

pub enum TextStyle {
    None,
    Shadow,
    Outline,
}

#[must_use = "Should call .draw() to draw the object"]
pub struct Text<'draw, 's, T: Draw> {
    draw: &'draw mut T,
    origin: Point,
    text: &'s str,
    color: Color,
    font: Font,
    font_size: f32,
    style: TextStyle,
    shadow_color: Color,
}

impl<'draw, 'b, T: Draw> Text<'draw, 'b, T> {
    setter! {font, Font}
    setter! {font_size, f32}
    setter! {style, TextStyle}
    setter! {shadow_color, Color}

    pub fn draw(self) {
        let mut draw = |color, offset: (f32, f32)| {
            self.draw.draw_text((self.origin.0 + offset.0, self.origin.1 + offset.1), self.text, color, TextOptions { font: self.font.clone(), font_size: self.font_size });
        };

        match self.style {
            TextStyle::Shadow => {
                draw(self.shadow_color, (1.0, 1.0));
            }
            TextStyle::Outline => {
                draw(self.shadow_color, (1.0, 1.0));
                draw(self.shadow_color, (1.0, -1.0));
                draw(self.shadow_color, (-1.0, 1.0));
                draw(self.shadow_color, (-1.0, -1.0));
                draw(self.shadow_color, (0.0, 1.0));
                draw(self.shadow_color, (0.0, -1.0));
                draw(self.shadow_color, (1.0, 0.0));
                draw(self.shadow_color, (-1.0, 0.0));
            }
            TextStyle::None => {}
        }

        draw(self.color, (0., 0.))
    }
}

#[must_use = "Should call .draw() to draw the object"]
pub struct Line<'draw, T: Draw> {
    draw: &'draw mut T,
    p1: Point,
    p2: Point,
    color: Color,
    thickness: f32,
}

impl<'draw, T: Draw> Line<'draw, T> {
    setter! { thickness, f32 }

    pub fn draw(self) {
        self.draw.draw_line(self.p1, self.p2, self.color, LineOptions { thickness: self.thickness })
    }
}

#[must_use = "Should call .draw() to draw the object"]
pub struct Rect<'draw, T: Draw> {
    draw: &'draw mut T,
    p1: Point,
    p2: Point,
    color: Color,
    thickness: f32,
    rounding: f32,
    filled: bool,
}

impl<'draw, T: Draw> Rect<'draw, T> {
    setter! { thickness, f32 }
    setter! { filled, bool }

    pub fn draw(self) {
        match self.filled {
            false => self.draw.draw_rect(self.p1, self.p2, self.color, RectOptions { thickness: self.thickness, rounding: self.rounding }),
            true => self.draw.draw_rect_filled(self.p1, self.p2, self.color, RectOptions { thickness: self.thickness, rounding: self.rounding }),
        }
    }
}

#[must_use = "Should call .draw() to draw the object"]
pub struct Circle<'draw, T: Draw> {
    draw: &'draw mut T,
    origin: Point,
    radius: f32,
    thickness: f32,
    color: Color,
    filled: bool,
}

impl<'draw, T: Draw> Circle<'draw, T> {
    setter! { filled, bool }
    setter! { thickness, f32 }

    pub fn draw(self) {
        match self.filled {
            false => self.draw.draw_circle(self.origin, self.radius, self.color, CircleOptions { thickness: self.thickness }),
            true => self.draw.draw_circle_filled(self.origin, self.radius, self.color, CircleOptions { thickness: self.thickness }),
        }
    }
}

pub struct NullFrame;

#[allow(unused_variables)]
impl Draw for NullFrame {
    fn draw_text(&mut self, origin: Point, text: &str, color: Color, options: TextOptions) {}

    fn draw_line(&mut self, p1: Point, p2: Point, color: Color, options: LineOptions) {}

    fn draw_rect(&mut self, p1: Point, p2: Point, color: Color, options: RectOptions) {}

    fn draw_rect_filled(&mut self, p1: Point, p2: Point, color: Color, options: RectOptions) {}

    fn draw_circle(&mut self, origin: Point, radius: f32, color: Color, options: CircleOptions) {}

    fn draw_circle_filled(&mut self, origin: Point, radius: f32, color: Color, options: CircleOptions) {}
}

pub struct NullOverlay(NullFrame);

impl Default for NullOverlay {
    fn default() -> Self {
        Self(NullFrame {})
    }
}

impl Render for NullOverlay {
    type Frame = NullFrame;

    fn add_custom_font(&mut self, font_data: Vec<u8>, font_size: f32, id: FontId) -> Option<()> {
        Some(())
    }

    fn frame_size(&self) -> (u32, u32) {
        (0, 0)
    }

    fn frame(&mut self) -> &mut Self::Frame {
        &mut self.0
    }

    fn render(&mut self) {}
}