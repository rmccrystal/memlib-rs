use core::ptr::null_mut;
use imgui_dx9_renderer::Renderer;
use winapi::shared::d3d9::{LPDIRECT3DDEVICE9, IDirect3DDevice9};
use winapi::shared::windef::{HWND, RECT};
use winapi::um::winuser::{DispatchMessageA, MSG, PeekMessageA, PM_REMOVE, TranslateMessage, GetClientRect, ShowWindow, SW_SHOWDEFAULT, UpdateWindow, GetWindowLongA, GWL_STYLE, WS_POPUP, WS_CLIPSIBLINGS, WS_DISABLED, WS_VISIBLE, SetWindowLongA, GetWindow, GetForegroundWindow, GW_HWNDPREV, SetWindowPos, SWP_ASYNCWINDOWPOS, SWP_NOMOVE, SWP_NOSIZE};
use imgui::{Context, Ui, DrawData, ImColor};
use super::util;
use crate::memory::Result;
use winapi::um::profileapi::{QueryPerformanceCounter, QueryPerformanceFrequency};
use imgui::FontSource::DefaultFontData;
use crate::overlay::Color;

pub struct Imgui {
    pub context: Context,
    pub renderer: Renderer,
    pub device: &'static mut IDirect3DDevice9,
    window: HWND,
    ticks_per_second: i64,
    time: i64,
}

impl Imgui {
    pub fn from_dx9(window: HWND) -> Result<Imgui> {
        let device = unsafe { util::create_d3d_device(window)? };

        let mut context = imgui::Context::create();
        let renderer = unsafe {
            imgui_dx9_renderer::Renderer::new_raw(&mut context, device)
        }.unwrap();

        let mut ticks_per_second: i64 = 0;
        let mut time: i64 = 0;

        unsafe {
            QueryPerformanceFrequency(&mut ticks_per_second as *mut _ as _);
            QueryPerformanceCounter(&mut time as *mut _ as _);
        }

        Ok(Self {
            renderer,
            context,
            device: unsafe { device.as_mut().unwrap() },
            window,
            ticks_per_second,
            time,
        })
    }

    pub fn main_loop(&mut self, mut run_ui: impl FnMut(&mut Ui) + 'static) {
        let mut msg = unsafe { std::mem::zeroed() };
        loop {
            unsafe {
                let style = (WS_POPUP | WS_CLIPSIBLINGS | WS_DISABLED | WS_VISIBLE) as i32;
                if GetWindowLongA(self.window, GWL_STYLE) != style {
                    SetWindowLongA(self.window, GWL_STYLE, style);
                }

                let foreground_window = GetWindow(GetForegroundWindow(), GW_HWNDPREV);
                if foreground_window != self.window {
                    SetWindowPos(self.window, foreground_window, 0, 0, 0, 0, SWP_ASYNCWINDOWPOS | SWP_NOMOVE | SWP_NOSIZE);
                    UpdateWindow(self.window);
                }

                if PeekMessageA(&mut msg, self.window, 0, 0, PM_REMOVE) > 0 {
                    TranslateMessage(&msg);
                    DispatchMessageA(&msg);
                    continue;
                }
            }
            unsafe {
                let io = self.context.io_mut();

                let mut rect: RECT = Default::default();
                GetClientRect(self.window, &mut rect);
                io.display_size = [(rect.right - rect.left) as _, (rect.bottom - rect.top) as _];

                let mut current_time: i64 = 0;
                QueryPerformanceCounter(&mut current_time as *mut _ as _);
                io.delta_time = (current_time - self.time) as f32 / self.ticks_per_second as f32;
                self.time = current_time;
            }

            let mut ui = self.context.frame();

            run_ui(&mut ui);

            let draw_data = ui.render();
            unsafe { self.device.BeginScene() };
            self.renderer.render(&draw_data).unwrap();
            unsafe { self.device.EndScene() };
        }
    }

    pub fn render(&mut self, draw_data: &DrawData) {

        // let mut current_time = Default::default();
        // unsafe { QueryPerformanceCounter(&mut current_time)};
        // io.delta_time = (current_time - )

        self.renderer.render(&draw_data).unwrap();
    }
}
