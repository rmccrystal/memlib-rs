
use crate::overlay::imgui::Imgui;
use crate::overlay::{Color, Draw, Font, LineOptions, TextOptions, TextStyle};

use imgui::*;
use crate::memory::Handle;
use crate::memory::handle_interfaces::driver_handle::DriverProcessHandle;
use std::fs::File;
use std::io::Write;
use crate::memory::handle_interfaces::winapi_handle::WinAPIProcessHandle;


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
    // let handle = Handle::from_interface(DriverProcessHandle::attach("notepad.exe").unwrap());
    let handle = Handle::from_interface(WinAPIProcessHandle::attach("notepad.exe").unwrap());
    let module = handle.get_module("notepad.exe").unwrap();
    println!("base: {:X}, size: {:X}", module.base_address, module.size);
    let dump = handle.dump_memory(module.get_memory_range());
    File::create("dump.exe").unwrap().write_all(&dump).unwrap();
}

fn _main() {
    let window = overlay::window::Window::hijack_nvidia().unwrap();
    let mut imgui = Imgui::from_window(window).unwrap();

    let _opened = true;
    imgui.main_loop(move |ui, _ctx| {
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
