use crate::math::Vector2;
use crate::overlay::imgui::Imgui;
use crate::overlay::{BoxOptions, Color, Draw, Font, LineOptions, TextOptions, TextStyle};
use crate::util::LoopTimer;
use imgui::*;
use std::ptr::null_mut;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::winuser::{CreateWindowExA, WNDCLASSEXA};
use imgui::NavInput::FocusNext;

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
    let window = overlay::window::Window::hijack_nvidia().unwrap();
    let mut imgui = Imgui::from_window(window).unwrap();

    let mut opened = true;
    imgui.main_loop(move |ui, ctx| {
        Window::new(im_str!("test"))
            .build(&ui, || {
                ui.text(ui.io().framerate.to_string());
            })
    }, move |overlay| {
        overlay.draw_line(overlay.ui.io().mouse_pos.into(), (0, 0).into(), LineOptions::default().color(Color::rose5()).width(15.0));
        esp(overlay);
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
        "test",
        TextOptions::default()
            .font(Font::Verdana)
            .style(TextStyle::Shadow)
            .color(Color::from_rgb(255, 255, 255)),
    );
}
