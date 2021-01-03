use crate::overlay::nvidia::NvidiaOverlay;
use crate::overlay::{OverlayInterface, BoxOptions, Color, TextOptions, Font, TextStyle};
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

        overlay.draw_text(Vector2 { x: 300.0, y: 200.0 }, "test123", TextOptions::default()
            .color(Color::white())
            .font(Font::Tahoma)
            .style(TextStyle::Shadow)
            .font_size(Some(20.0))
        );

        overlay.draw_text(Vector2 { x: 200.0, y: 200.0 }, "DOasdfWN", TextOptions::default()
            .color(Color::from_rgb(255, 0, 0))
            .font(Font::Pixel)
            .style(TextStyle::Outlined)
            .font_size(Some(10.0))
        );

        overlay.end();
    }
}
