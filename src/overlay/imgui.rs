use core::ptr::null_mut;

use anyhow::Result;
use imgui::{Context, DrawData, ImColor, Ui};
use imgui::FontSource::DefaultFontData;
use imgui_dx9_renderer::Renderer;
use log::*;
use winapi::_core::ptr::null;
use winapi::shared::d3d9::{IDirect3DDevice9, LPDIRECT3DDEVICE9};
use winapi::shared::d3d9types::{D3DCLEAR_TARGET, D3DCLEAR_ZBUFFER, D3DRS_ALPHABLENDENABLE, D3DRS_SCISSORTESTENABLE, D3DRS_ZENABLE};
use winapi::shared::ntdef::FALSE;
use winapi::shared::windef::{HWND, RECT};
use winapi::um::profileapi::{QueryPerformanceCounter, QueryPerformanceFrequency};
use winapi::um::winuser::{DispatchMessageA, GetClientRect, GetForegroundWindow, GetWindow, GetWindowLongA, GW_HWNDPREV, GWL_STYLE, MSG, PeekMessageA, PM_REMOVE, SetWindowLongA, SetWindowPos, ShowWindow, SW_SHOWDEFAULT, SWP_ASYNCWINDOWPOS, SWP_NOMOVE, SWP_NOSIZE, TranslateMessage, UpdateWindow, WS_CLIPSIBLINGS, WS_DISABLED, WS_POPUP, WS_VISIBLE};

use crate::overlay::Color;
use crate::overlay::util::D3DDevice9;

use super::util;
use super::util::ToError;

pub struct Imgui {
    pub context: Context,
    pub renderer: Renderer,
    pub device: D3DDevice9,
    window: HWND,
    ticks_per_second: i64,
    time: i64,
}

impl Imgui {
    pub fn from_window(window: HWND) -> Result<Imgui> {
        let mut device = unsafe { util::create_d3d_device(window)? };

        let mut context = imgui::Context::create();
        let renderer = unsafe {
            imgui_dx9_renderer::Renderer::new_raw(&mut context, &mut *device)
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
            device,
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
            unsafe {
                self.device.BeginScene().to_err().unwrap();

                self.renderer.render(&draw_data).unwrap();

                self.device.EndScene().to_err().unwrap();
                if let Err(e) = self.device.Present(null(), null(), null_mut(), null()).to_err() {
                    error!("Erorr in DX9 Present for IMGUI: {:?}", e);
                }
            }
        }
    }
}
