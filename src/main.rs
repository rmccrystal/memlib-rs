#![feature(asm)]

use crate::overlay::imgui::{Imgui, ImguiConfig};
use crate::overlay::{Color, Draw, Font, LineOptions, TextOptions, TextStyle};

use imgui::*;
use crate::memory::Handle;

use std::fs::File;
use std::io::Write;
use crate::memory::handle_interfaces::winapi_handle::WinAPIProcessHandle;
use crate::memory::handle_interfaces::driver_handle::DriverProcessHandle;
use crate::logger::MinimalLogger;
use log::LevelFilter;
use crate::winutil::{get_pid_by_name, inject_func, InputEventListener, Event};
use winapi::um::processthreadsapi::GetCurrentProcessId;
use win_key_codes::VK_INSERT;


pub mod hacks;
pub mod logger;
pub mod math;
pub mod memory;
pub mod overlay;
pub mod system;
pub mod util;
pub mod winutil;

#[macro_use]
pub mod macros;

fn _asdmain() {
    let handle = Handle::from_interface(DriverProcessHandle::attach("notepad.exe").unwrap());
    // let handle = Handle::from_interface(WinAPIProcessHandle::attach("notepad.exe").unwrap());
    let module = handle.get_module("notepad.exe").unwrap();
    println!("base: {:X}, size: {:X}", module.base_address, module.size);

    let dump = handle.dump_memory(module.get_memory_range());
    dbg!(dump[0]);
    // File::create("dump.exe").unwrap().write_all(&dump).unwrap();
}

struct Data {

}

extern "C" fn func(num: *mut u32) -> u32 {
    unsafe { *num }
}

fn _main() {
    MinimalLogger::init(LevelFilter::Trace);

    // let pid = get_pid_by_name("notepad.exe").unwrap();
    let pid = unsafe {GetCurrentProcessId()};

    let num = 1;
    // let status = inject_func(pid, func, &num).unwrap();

    // println!("status: {:X}", status);
}

fn n_main() {
    use win_key_codes::*;
    let lis = InputEventListener::new();
}

fn main() {
    MinimalLogger::init(LevelFilter::Trace).unwrap();

    let window = overlay::window::Window::hijack_nvidia().unwrap();
    let mut imgui = Imgui::from_window(window, ImguiConfig{toggle_menu_key: Some(VK_INSERT)}).unwrap();

    let _opened = true;
    imgui.main_loop(move |ui, _ctx| {
        Window::new(im_str!("test"))
            .build(&ui, || {
                ui.text(ui.io().framerate.to_string());
                ui.button(im_str!("button"), [100.0, 200.0]);
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
