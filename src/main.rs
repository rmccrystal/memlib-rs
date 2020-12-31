use crate::overlay::nvidia::NvidiaOverlay;
use crate::overlay::{OverlayInterface, BoxOptions, Color, TextOptions, Font};
use crate::util::LoopTimer;
use crate::math::Vector2;

pub mod hacks;
pub mod logger;
pub mod math;
pub mod memory;
pub mod util;
pub mod system;
pub mod overlay;

#[macro_use]
pub mod macros;

// There are going to be different error types throughout
// this package, so define Result to use a boxed Error trait
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    // let handle = memory::Handle::from(
    //     memory::handle_interfaces::driver_handle::DriverProcessHandle::attach("notepad.exe").unwrap()
    // );
    //
    // println!("{:?}", handle.read_memory::<u32>(1000000000));

    let mut overlay = NvidiaOverlay::init().unwrap();
    let mut timer = LoopTimer::new(50);

    // overlay.begin();overlay.end();return;
    loop {
        timer.wait();
        overlay.begin();

        overlay.draw_text(Vector2 { x: 100.0, y: 200.0 }, "hello", TextOptions::default()
            .color(Color::white())
            .font(Font::Pixel)
            .font_size(Some(20.0))
        );

        overlay.end();
    }
}
