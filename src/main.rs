use crate::overlay::nvidia::NvidiaOverlay;
use crate::overlay::{OverlayInterface, BoxOptions, Color};
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
    let mut timer = LoopTimer::new(1);

    // overlay.begin(); overlay.end(); return;
    loop {
        timer.wait();
        overlay.begin();

        overlay.draw_box(
            Vector2{x: 50.0, y: 100.0 },
            Vector2{x: 150.0, y: 200.0 },
            BoxOptions::default()
                .color(Color::from_rgba(255, 0, 0, 100))
        );

        overlay.end();

        timer.wait();
        overlay.begin();

        overlay.draw_box(
            Vector2{x: 500.0, y: 1000.0 },
            Vector2{x: 1500.0, y: 200.0 },
            BoxOptions::default()
                .color(Color::from_rgba(255, 0, 0, 100))
        );

        overlay.end();

    }
}
