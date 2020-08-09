use overlay::Color;
use overlay::OverlayInterface;
use std::thread::sleep;
use math::Vector2;
use overlay::{TextStyle, Font};

pub mod hacks;
pub mod logger;
pub mod math;
pub mod memory;
pub mod util;
pub mod system;
pub mod overlay;

#[macro_use]
pub mod macros;

fn main() {
    let mut ov = overlay::looking_glass::LookingGlassOverlay::new("/tmp/overlay-pipe", 0).unwrap();
    // let mut ov = overlay::looking_glass::LookingGlassOverlay::new("/tmp/test", 0).unwrap();
    println!("Created overlay");
    loop {
        ov.begin();
        ov.draw_text(
            Vector2{x: 100.0, y: 200.0},
            "hello".to_string(),
            Color::from_rgb(255, 255, 255),
            TextStyle::Shadow,
            Font::Verdana,
            0.0
        );
        ov.end();
        sleep(std::time::Duration::from_millis(1000));
    }
}
