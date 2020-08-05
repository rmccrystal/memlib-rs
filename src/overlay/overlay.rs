use super::Color;

pub struct Overlay {
    inner: Box<dyn OverlayInterface>
}

impl Overlay {
    pub fn from_interface(interface: impl OverlayInterface + 'static) -> Self {
        Self {
            inner: Box::new(interface)
        }
    }
}

impl OverlayInterface for Overlay {
    fn draw_line(&mut self, p1: (i32, i32), p2: (i32, i32), color: Color, width: i32) {
        self.inner.draw_line(p1, p2, color, width)
    }

    fn draw_box(&mut self, p1: (i32, i32), p2: (i32, i32), color: Color, thickness: i32) {
        self.inner.draw_box(p1, p2, color, thickness)
    }

    fn draw_box_filled(&mut self, p1: (i32, i32), p2: (i32, i32), color: Color) {
        self.inner.draw_box_filled(p1, p2, color)
    }

    fn draw_text(&mut self, origin: (i32, i32), text: String, color: Color, size: u8) {
        self.inner.draw_text(origin, text, color, size)
    }
}

pub trait OverlayInterface {
    fn draw_line(&mut self, p1: (i32, i32), p2: (i32, i32), color: Color, width: i32);
    fn draw_box(&mut self, p1: (i32, i32), p2: (i32, i32), color: Color, thickness: i32);
    fn draw_box_filled(&mut self, p1: (i32, i32), p2: (i32, i32), color: Color);
    fn draw_text(&mut self, origin: (i32, i32), text: String, color: Color, size: u8);
}