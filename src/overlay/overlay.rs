use super::Color;
use crate::math::Vector2;
use crate::overlay::TextStyle;

pub type Overlay = Box<dyn OverlayInterface + Sync + Send + 'static>;

pub trait OverlayInterface {
    fn begin(&mut self);
    fn end(&mut self);
    fn draw_line(&mut self, p1: Vector2, p2: Vector2, color: Color, width: f32);
    fn draw_box(&mut self, p1: Vector2, p2: Vector2, color: Color, width: f32, rounding: f32, filled: bool);
    fn draw_text(&mut self, origin: Vector2, text: &str, color: Color, style: TextStyle, font: super::Font, font_size: f32, centered: bool);
}