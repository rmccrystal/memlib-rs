use crate::math::Vector2;
use crate::overlay::imgui::Imgui;
use crate::overlay::{BoxOptions, Color, Draw, Font, LineOptions, TextOptions, TextStyle};
use crate::util::LoopTimer;
use imgui::*;
use std::ptr::null_mut;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::winuser::{CreateWindowExA, WNDCLASSEXA};

pub mod hacks;
pub mod logger;
pub mod math;
pub mod memory;
pub mod overlay;
pub mod system;
pub mod util;

#[macro_use]
pub mod macros;

fn main() {
    // let handle = memory::Handle::from(
    //     memory::handle_interfaces::driver_handle::DriverProcessHandle::attach("notepad.exe").unwrap()
    // );
    //
    // println!("{:?}", handle.read_memory::<u32>(1000000000));

    let window = unsafe {
        overlay::util::hijack_window("CEF-OSC-WIDGET", "NVIDIA GeForce Overlay").unwrap()
    };

    let mut imgui = Imgui::from_window(window).unwrap();

    imgui.main_loop(move |frame| {
        esp(frame);
        let ui = &mut frame.ui;

        let token = ui.push_font(*frame.font_ids.get(&Font::Verdana).unwrap());
        Window::new(im_str!("Hello world"))
            .size([300.0, 110.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.text(im_str!("Hello world!"));
                ui.text(im_str!("こんにちは世界！"));
                ui.text(im_str!("This...is...imgui-rs!"));
                ui.separator();
                let mouse_pos = ui.io().mouse_pos;
                ui.text(format!(
                    "Mouse Position: ({:.1},{:.1})",
                    mouse_pos[0], mouse_pos[1]
                ));
            });

        token.pop(&ui);
    })
}

pub fn esp(draw: &mut impl Draw) {
    draw.draw_line(
        (10.0, 10.0).into(),
        (100.0, 200.0).into(),
        LineOptions::default().width(20.0).color(Color::from_hex(0xFF0000FF)),
    );
    draw.draw_text(
        (400, 400).into(),
        "flashed",
        TextOptions::default()
            .font(Font::Verdana)
            .style(TextStyle::None)
            .color(Color::from_rgb(255, 255, 255)),
    );
}
