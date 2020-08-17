
use crate::math::Vector2;

use super::commands::*;

pub type Overlay = Box<dyn OverlayInterface + Sync + Send + 'static>;

pub trait OverlayInterface {
    fn begin(&mut self);
    fn end(&mut self);
    fn draw_line(&mut self, p1: Vector2, p2: Vector2, options: LineOptions);
    fn draw_box(&mut self, p1: Vector2, p2: Vector2, options: BoxOptions);
    fn draw_text(&mut self, origin: Vector2, text: &str, options: TextOptions);
    fn draw_circle(&mut self, origin: Vector2, radius: f32, options: CircleOptions);
}