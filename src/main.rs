use crate::overlay::{OverlayInterface, BoxOptions, Color, TextOptions, Font, TextStyle};
use crate::util::LoopTimer;
use crate::math::Vector2;
use crate::overlay::imgui::Imgui;
use imgui::*;
use winapi::um::winuser::{WNDCLASSEXA, CreateWindowExA};
use winapi::um::libloaderapi::GetModuleHandleA;
use std::ptr::null_mut;

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

    let window = unsafe { overlay::util::hijack_window("CEF-OSC-WIDGET", "NVIDIA GeForce Overlay").unwrap() };
    let mut imgui = Imgui::from_window(window).unwrap();

    imgui.main_loop(move |ui| {
        {
            let bg_draw_list = ui.get_background_draw_list();
            bg_draw_list
                .add_circle([150.0, 150.0], 150.0, [1.0, 0.0, 0.0])
                .thickness(4.0)
                .build();
        }

        {
            let [w, h] = ui.io().display_size;
            let fg_draw_list = ui.get_foreground_draw_list();
            fg_draw_list
                .add_circle([w - 150.0, h - 150.0], 150.0, [1.0, 0.0, 0.0])
                .thickness(4.0)
                .build();
        }

        Window::new(im_str!("Draw list"))
            .size([300.0, 110.0], Condition::FirstUseEver)
            .scroll_bar(false)
            .build(ui, || {
                ui.button(im_str!("random button"), [0.0, 0.0]);
                let draw_list = ui.get_window_draw_list();
                let o = ui.cursor_screen_pos();
                let ws = ui.content_region_avail();
                draw_list
                    .add_circle([o[0] + 10.0, o[1] + 10.0], 5.0, [1.0, 0.0, 0.0])
                    .thickness(4.0)
                    .build();
                draw_list
                    .add_circle([o[0] + ws[0] - 10.0, o[1] + 10.0], 5.0, [0.0, 1.0, 0.0])
                    .thickness(4.0)
                    .build();
                draw_list
                    .add_circle(
                        [o[0] + ws[0] - 10.0, o[1] + ws[1] - 10.0],
                        5.0,
                        [0.0, 0.0, 1.0],
                    )
                    .thickness(4.0)
                    .build();
                draw_list
                    .add_circle([o[0] + 10.0, o[1] + ws[1] - 10.0], 5.0, [1.0, 1.0, 0.0])
                    .thickness(4.0)
                    .build();
                draw_text_centered(
                    ui,
                    &draw_list,
                    [o[0], o[1], ws[0], ws[1]],
                    im_str!("window draw list"),
                    [1.0, 1.0, 1.0],
                );
            });
    })
}
