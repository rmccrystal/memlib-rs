use core::ptr::null_mut;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use anyhow::Result;
use imgui::sys::{ImDrawList, ImFont_CalcTextSizeA, ImVec2};
use imgui::*;
use imgui_dx9_renderer::Renderer;
use log::*;
use winapi::_core::ptr::null;
use winapi::shared::d3d9::{IDirect3DDevice9, LPDIRECT3DDEVICE9};
use winapi::shared::d3d9types::{
    D3DCLEAR_TARGET, D3DCLEAR_ZBUFFER, D3DRS_ALPHABLENDENABLE, D3DRS_SCISSORTESTENABLE,
    D3DRS_ZENABLE,
};
use winapi::shared::ntdef::FALSE;
use winapi::shared::windef::{HWND, RECT, POINT};
use winapi::um::profileapi::{QueryPerformanceCounter, QueryPerformanceFrequency};
use winapi::um::winuser::{DispatchMessageA, GetClientRect, GetForegroundWindow, GetWindow, GetWindowLongA, PeekMessageA, SetWindowLongA, SetWindowPos, ShowWindow, TranslateMessage, UpdateWindow, GWL_STYLE, GW_HWNDPREV, MSG, PM_REMOVE, SWP_ASYNCWINDOWPOS, SWP_NOMOVE, SWP_NOSIZE, SW_SHOWDEFAULT, WS_CLIPSIBLINGS, WS_DISABLED, WS_POPUP, WS_VISIBLE, IsChild, GetCursorPos, ScreenToClient};

use crate::math::Vector2;
use crate::overlay::imgui::fonts::create_fonts;
use crate::overlay::util::{create_d3d_device, D3DDevice9};
use crate::overlay::{BoxOptions, CircleOptions, Color, Draw, LineOptions, TextOptions, TextStyle};

use super::util::ToError;
use winapi::shared::minwindef::TRUE;

mod fonts;

pub struct Imgui {
    context: Context,
    renderer: Renderer,
    device: D3DDevice9,
    window: HWND,
    ticks_per_second: i64,
    time: i64,
    font_ids: HashMap<super::types::Font, FontId>,
}

impl Deref for Imgui {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl DerefMut for Imgui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}

impl Imgui {
    pub fn from_window(window: HWND) -> Result<Imgui> {
        let mut device = unsafe { create_d3d_device(window)? };

        let mut context = imgui::Context::create();

        let font_ids = create_fonts(&mut context);

        Self::init_style(context.style_mut());

        let renderer =
            unsafe { imgui_dx9_renderer::Renderer::new_raw(&mut context, &mut *device) }.unwrap();

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
            font_ids,
        })
    }

    fn init_style(style: &mut Style) {
        style.anti_aliased_lines = true;
        style.anti_aliased_fill = true;
        style.anti_aliased_lines_use_tex = true;
        // style.alpha = 0.2;
    }

    fn update_mouse_pos(&mut self) {
        unsafe {
            let foreground_window = GetForegroundWindow();
            if foreground_window.is_null() {
                return;
            }

            // if (foreground_window == self.window) || (IsChild(foreground_window, self.window) == TRUE) {
                println!("here");
                let mut pos: POINT = std::mem::zeroed();
                if GetCursorPos(&mut pos) == TRUE && ScreenToClient(self.window, &mut pos) == TRUE {
                    self.io_mut().mouse_pos = [pos.x as _, pos.y as _];
                } else {
                    warn!("Could not get cursor position for IMGUI");
                }
            // }
        }
    }

    fn update_window(&mut self) {
        let mut msg = unsafe { std::mem::zeroed() };
        unsafe {
            let style = (WS_POPUP | WS_CLIPSIBLINGS | WS_DISABLED | WS_VISIBLE) as i32;
            if GetWindowLongA(self.window, GWL_STYLE) != style {
                SetWindowLongA(self.window, GWL_STYLE, style);
            }

            let foreground_window = GetWindow(GetForegroundWindow(), GW_HWNDPREV);
            if foreground_window != self.window {
                SetWindowPos(
                    self.window,
                    foreground_window,
                    0,
                    0,
                    0,
                    0,
                    SWP_ASYNCWINDOWPOS | SWP_NOMOVE | SWP_NOSIZE,
                );
                UpdateWindow(self.window);
            }

            if PeekMessageA(&mut msg, self.window, 0, 0, PM_REMOVE) > 0 {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }

            let io = self.context.io_mut();

            let mut rect: RECT = Default::default();
            GetClientRect(self.window, &mut rect);
            io.display_size = [(rect.right - rect.left) as _, (rect.bottom - rect.top) as _];

            let mut current_time: i64 = 0;
            QueryPerformanceCounter(&mut current_time as *mut _ as _);
            io.delta_time = (current_time - self.time) as f32 / self.ticks_per_second as f32;
            self.time = current_time;
        }
    }

    fn present(device: &D3DDevice9, renderer: &mut Renderer, draw_data: &DrawData) {
        unsafe {
            device.BeginScene().to_err().unwrap();

            renderer.render(&draw_data).unwrap();

            device.EndScene().to_err().unwrap();
            if let Err(e) = device
                .Present(null(), null(), null_mut(), null())
                .to_err()
            {
                error!("Error in DX9 Present for IMGUI: {:?}", e);
            }
        }
    }

    pub fn main_loop(&mut self, mut run_ui: impl FnMut(&mut ImguiFrame) + 'static) {
        loop {
            self.update_window();
            self.update_mouse_pos();

            let draw_data = {
                let mut frame = ImguiFrame::new(&mut self.context, &self.font_ids);
                run_ui(&mut frame);
                frame.render()
            };

            Self::present(&self.device, &mut self.renderer, &draw_data);
        }
    }
}

/// Represents a frame that be can be drawn on
pub struct ImguiFrame<'ui> {
    pub ui: Ui<'ui>,
    pub font_ids: HashMap<super::types::Font, FontId>,
}

impl<'ui> ImguiFrame<'ui> {
    /// Creates a frame from a context
    pub fn new(
        ctx: &'ui mut imgui::Context,
        font_ids: &HashMap<super::types::Font, FontId>,
    ) -> Self {
        let ui = ctx.frame();
        Self {
            ui,
            font_ids: font_ids.clone(),
        }
    }

    pub fn render(self) -> &'ui DrawData {
        self.ui.render()
    }

    fn get_draw_list(&self) -> DrawListMut {
        self.ui.get_background_draw_list()
    }
}

// impl<'im> Deref for Frame<'im> {
//     type Target = Ui<'im>;
//
//     fn deref(&self) -> &Self::Target {
//         &self.ui
//     }
// }
//
// impl<'im> DerefMut for Frame<'im> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.ui
//     }
// }

impl Draw for ImguiFrame<'_> {
    fn draw_line(&mut self, p1: Vector2, p2: Vector2, options: LineOptions) {
        let draw_list = self.get_draw_list();
        draw_list
            .add_line(p1.into(), p2.into(), options.color)
            .thickness(options.width)
            .build()
    }

    fn draw_box(&mut self, p1: Vector2, p2: Vector2, options: BoxOptions) {
        let draw_list = self.get_draw_list();
        draw_list
            .add_rect(p1.into(), p2.into(), options.color)
            .thickness(options.width)
            .rounding(options.rounding)
            .filled(options.filled)
            .build()
    }

    fn draw_text(&mut self, origin: Vector2, text: &str, options: TextOptions) {
        let draw_list = self.get_draw_list();
        let text = unsafe { ImStr::from_ptr_unchecked(ImString::new(text).as_ptr()) };

        let font = *self.font_ids.get(&options.font).unwrap();
        let font_size = options.font_size.unwrap_or(10.0);

        let font_token = self.ui.push_font(font);

        let x = match options.centered_horizontal {
            false => origin.x,
            true => origin.x + self.ui.calc_text_size(text, false, 0.0)[0],
        };
        let y = match options.centered_horizontal {
            false => origin.y,
            true => origin.y + self.ui.calc_text_size(text, false, 0.0)[0],
        };

        // let draw_list_addr: *mut ImDrawList = unsafe { std::mem::transmute(draw_list) };

        let draw = |color, offset: (f32, f32)| {
            draw_list.add_text([x + offset.0, y + offset.1], color, text);
        };

        let shadow_color = options.shadow_color;
        match options.style {
            TextStyle::Shadow => {
                draw(shadow_color, (1.0, 1.0));
            }
            TextStyle::Outlined => {
                draw(shadow_color, (1.0, 1.0));
                draw(shadow_color, (1.0, -1.0));
                draw(shadow_color, (-1.0, 1.0));
                draw(shadow_color, (-1.0, -1.0));
                draw(shadow_color, (0.0, 1.0));
                draw(shadow_color, (0.0, -1.0));
                draw(shadow_color, (1.0, 0.0));
                draw(shadow_color, (-1.0, 0.0));
            }
            TextStyle::None => {}
        }

        draw(options.color, (0.0, 0.0));

        font_token.pop(&self.ui);
    }

    fn draw_circle(&mut self, origin: Vector2, radius: f32, options: CircleOptions) {
        unimplemented!()
    }
}
