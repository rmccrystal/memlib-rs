use core::ptr::null_mut;
use std::collections::HashMap;


use anyhow::Result;

pub use imgui::*;
use imgui_dx9_renderer::Renderer;
use log::*;
use winapi::_core::ptr::null;

use winapi::shared::d3d9types::{D3DCLEAR_TARGET, D3DCLEAR_ZBUFFER, D3DRS_ALPHABLENDENABLE, D3DRS_SCISSORTESTENABLE, D3DRS_ZENABLE, D3DCOLOR_RGBA};

use winapi::um::profileapi::{QueryPerformanceCounter, QueryPerformanceFrequency};
use winapi::um::winuser::*;

use crate::math::Vector2;
use crate::overlay::imgui::fonts::create_fonts;
use crate::overlay::dx9::{create_d3d_device, D3DDevice9};
use crate::overlay::{BoxOptions, CircleOptions, Draw, LineOptions, TextOptions, TextStyle};

use crate::winutil::{ToError, InputEventListener, Event};
use winapi::shared::minwindef::TRUE;

use super::types;
use crate::winutil::is_key_down;
use super::window;
use winapi::shared::windef::{POINT, RECT};


mod fonts;

pub struct Imgui {
    pub context: Context,
    renderer: Renderer,
    device: D3DDevice9,
    window: window::Window,
    ticks_per_second: i64,
    time: i64,
    pub font_ids: HashMap<super::types::Font, FontId>,
    pub config: ImguiConfig,
    input_listener: InputEventListener,
    ui_enabled: bool,
    last_window_size: RECT
}

unsafe impl Send for Imgui{}
unsafe impl Sync for Imgui{}

#[derive(Default)]
pub struct ImguiConfig {
    pub toggle_menu_key: Option<i32>,
    pub align_to_pixel: bool
}

impl Imgui {
    pub fn from_window(window: window::Window, config: ImguiConfig) -> Result<Imgui> {
        let mut device = unsafe { create_d3d_device(&window)? };

        let mut context = imgui::Context::create();

        let font_ids = create_fonts(&mut context);

        Self::init_style(context.style_mut());
        Self::init_keymap(context.io_mut());

        let renderer =
            unsafe { imgui_dx9_renderer::Renderer::new_raw(&mut context, &mut *device) }.unwrap();

        let mut ticks_per_second: i64 = 0;
        let mut time: i64 = 0;
        unsafe {
            QueryPerformanceFrequency(&mut ticks_per_second as *mut _ as _);
            QueryPerformanceCounter(&mut time as *mut _ as _);
        }

        let input_listener = InputEventListener::new();

        let rect = window.get_rect();

        Ok(Self {
            renderer,
            context,
            device,
            window,
            ticks_per_second,
            time,
            font_ids,
            config,
            input_listener,
            ui_enabled: false,
            last_window_size: rect
        })
    }

    fn init_style(style: &mut Style) {
        style.anti_aliased_lines = true;
        style.anti_aliased_fill = true;
    }

    fn init_keymap(io: &mut Io) {
        use imgui::sys::*;

        io.key_map[ImGuiKey_Tab as usize] = VK_TAB as _;
        io.key_map[ImGuiKey_LeftArrow as usize] = VK_LEFT as _;
        io.key_map[ImGuiKey_RightArrow as usize] = VK_RIGHT as _;
        io.key_map[ImGuiKey_UpArrow as usize] = VK_UP as _;
        io.key_map[ImGuiKey_DownArrow as usize] = VK_DOWN as _;
        io.key_map[ImGuiKey_PageUp as usize] = VK_PRIOR as _;
        io.key_map[ImGuiKey_PageDown as usize] = VK_NEXT as _;
        io.key_map[ImGuiKey_Home as usize] = VK_HOME as _;
        io.key_map[ImGuiKey_End as usize] = VK_END as _;
        io.key_map[ImGuiKey_Insert as usize] = VK_INSERT as _;
        io.key_map[ImGuiKey_Delete as usize] = VK_DELETE as _;
        io.key_map[ImGuiKey_Backspace as usize] = VK_BACK as _;
        io.key_map[ImGuiKey_Space as usize] = VK_SPACE as _;
        io.key_map[ImGuiKey_Enter as usize] = VK_RETURN as _;
        io.key_map[ImGuiKey_Escape as usize] = VK_ESCAPE as _;
        io.key_map[ImGuiKey_KeyPadEnter as usize] = VK_RETURN as _;
        io.key_map[ImGuiKey_A as usize] = 'A' as _;
        io.key_map[ImGuiKey_C as usize] = 'C' as _;
        io.key_map[ImGuiKey_V as usize] = 'V' as _;
        io.key_map[ImGuiKey_X as usize] = 'X' as _;
        io.key_map[ImGuiKey_Y as usize] = 'Y' as _;
        io.key_map[ImGuiKey_Z as usize] = 'Z' as _;
    }

    fn update_mouse_pos(&mut self) {
        unsafe {
            let mut pos: POINT = std::mem::zeroed();
            if GetCursorPos(&mut pos) == TRUE && ScreenToClient(self.window.hwnd, &mut pos) == TRUE {
                self.context.io_mut().mouse_pos = [pos.x as _, pos.y as _];
            } else {
                warn!("Could not get cursor position for IMGUI");
            }
        }
    }

    fn update_cursor(ui: &Ui) {
        if ui.io().config_flags.contains(ConfigFlags::NO_MOUSE_CURSOR_CHANGE) {
            return;
        }

        use MouseCursor::*;
        use winapi::um::winuser::*;

        let imgui_cursor = ui.mouse_cursor().unwrap_or(Arrow);

        let win32_cursor = match imgui_cursor {
            Arrow => IDC_ARROW,
            TextInput => IDC_IBEAM,
            ResizeAll => IDC_SIZEALL,
            ResizeEW => IDC_SIZEWE,
            ResizeNS => IDC_SIZENS,
            ResizeNESW => IDC_SIZENESW,
            ResizeNWSE => IDC_SIZENWSE,
            Hand => IDC_HAND,
            NotAllowed => IDC_NO
        };

        unsafe { SetCursor(win32_cursor as _) };
    }

    fn update_keyboard(&mut self) {
        let io = self.context.io_mut();

        io.key_ctrl = is_key_down(VK_CONTROL);
        io.key_shift = is_key_down(VK_SHIFT);
        io.key_alt = is_key_down(VK_MENU);
        io.key_super = false;

        io.mouse_down[0] = is_key_down(VK_LBUTTON);
        io.mouse_down[1] = is_key_down(VK_RBUTTON);
        io.mouse_down[2] = is_key_down(VK_MBUTTON);
    }

    fn update_window(&mut self) {
        unsafe {
            self.window.tick();

            let io = self.context.io_mut();

            let rect = self.window.get_rect();
            io.display_size = [(rect.right - rect.left) as _, (rect.bottom - rect.top) as _];

            let mut current_time: i64 = 0;
            QueryPerformanceCounter(&mut current_time as *mut _ as _);
            io.delta_time = (current_time - self.time) as f32 / self.ticks_per_second as f32;
            self.time = current_time;
        }
    }

    fn present(device: &D3DDevice9, renderer: &mut Renderer, draw_data: &DrawData) {
        unsafe {
            device.SetRenderState(D3DRS_ZENABLE, 0);
            device.SetRenderState(D3DRS_ALPHABLENDENABLE, 0);
            device.SetRenderState(D3DRS_SCISSORTESTENABLE, 0);
            let clear_color = D3DCOLOR_RGBA(0, 0, 0, 0);
            device.Clear(0, null(), D3DCLEAR_TARGET | D3DCLEAR_ZBUFFER, clear_color, 1.0, 0);

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

    fn update_keybinds(&mut self) {
        if let None = self.config.toggle_menu_key {
            return;
        }
        for event in &self.input_listener {
            if let Event::KeyDown(key) = event {
                if key == self.config.toggle_menu_key.unwrap() {
                    self.ui_enabled = !self.ui_enabled
                }
            }
        }
    }

    fn update_directx(&mut self) {
        let rect = self.window.get_rect();
        let last_rect = &self.last_window_size;
        if !((rect.top == last_rect.top)
            && (rect.bottom == last_rect.bottom)
            && (rect.right == last_rect.right)
            && (rect.left == last_rect.left))
        {
            // update device
            self.device = unsafe { create_d3d_device(&self.window).unwrap() };
            self.renderer = unsafe { imgui_dx9_renderer::Renderer::new_raw(&mut self.context, &mut *self.device) }.unwrap();
            self.last_window_size = rect;
        }
    }

    pub fn main_loop(&mut self, mut run_ui: impl FnMut(&mut Ui, &RenderContext), mut run_overlay: impl FnMut(&mut OverlayWindow)) {
        loop {
            self.render(&mut run_ui, &mut run_overlay);
        }
    }

    pub fn render(&mut self, run_ui: &mut impl FnMut(&mut Ui, &RenderContext), run_overlay: &mut impl FnMut(&mut OverlayWindow)) {
        self.update_window();
        self.update_directx();
        self.update_mouse_pos();
        self.update_keyboard();
        self.update_keybinds();

        let draw_data = {
            let context = RenderContext {
                font_ids: &self.font_ids
            };
            let mut ui = self.context.frame();

            self.window.set_clickthrough(!self.ui_enabled);
            if self.ui_enabled {
                run_ui(&mut ui, &context);
            }

            let mut overlay_window = OverlayWindow::begin(&mut ui, &self.font_ids, self.config.align_to_pixel);
            run_overlay(&mut overlay_window);
            overlay_window.end();

            Self::update_cursor(&ui);

            ui.render()
        };

        Self::present(&self.device, &mut self.renderer, &draw_data);
    }
}

pub struct RenderContext<'a> {
    font_ids: &'a HashMap<super::types::Font, FontId>,
}

impl RenderContext<'_> {
    pub fn get_font(&self, font: types::Font) -> FontId {
        *self.font_ids.get(&font).unwrap()
    }
}

/// Represents a frame that be can be drawn on
pub struct OverlayWindow<'a, 'ui> {
    pub font_ids: HashMap<super::types::Font, FontId>,
    pub ui: &'a mut Ui<'ui>,
    style_token: StyleStackToken,
    color_token: ColorStackToken,
    window_token: WindowToken,
    align_to_pixel: bool
}

impl<'a, 'ui> OverlayWindow<'a, 'ui> {
    /// Creates a frame from a context
    pub fn begin(
        ui: &'a mut Ui<'ui>,
        font_ids: &HashMap<super::types::Font, FontId>,
        align_to_pixel: bool
    ) -> Self {
        let style_token = ui.push_style_vars(&[StyleVar::WindowBorderSize(0.0), StyleVar::WindowPadding([0.0, 0.0])]);
        let color_token = ui.push_style_color(StyleColor::WindowBg, [0.0, 0.0, 0.0, 0.0]);
        let window_token = Window::new(im_str!("##overlay"))
            .flags(WindowFlags::NO_TITLE_BAR | WindowFlags::NO_INPUTS)
            .position([0.0, 0.0], Condition::Always)
            .size(ui.io().display_size, Condition::Always)
            .begin(&ui).unwrap();
        Self {
            font_ids: font_ids.clone(),
            style_token,
            window_token,
            color_token,
            ui,
            align_to_pixel
        }
    }

    pub fn end(self) {
        self.window_token.end(&self.ui);
        self.style_token.pop(&self.ui);
        self.color_token.pop(&self.ui);
    }

    fn get_draw_list(&self) -> WindowDrawList {
        self.ui.get_window_draw_list()
    }
}

impl Draw for OverlayWindow<'_, '_> {
    fn draw_line(&mut self, mut p1: Vector2, mut p2: Vector2, options: LineOptions) {
        if self.align_to_pixel {
            p1 = p1.round();
            p2 = p2.round();
        }

        let draw_list = self.get_draw_list();
        draw_list
            .add_line(p1.into(), p2.into(), options.color)
            .thickness(options.width)
            .build()
    }

    fn draw_box(&mut self, mut p1: Vector2, mut p2: Vector2, options: BoxOptions) {
        if self.align_to_pixel {
            p1 = p1.round();
            p2 = p2.round();
        }

        let draw_list = self.get_draw_list();
        draw_list
            .add_rect(p1.into(), p2.into(), options.color)
            .thickness(options.width)
            .rounding(options.rounding)
            .filled(options.filled)
            .build()
    }

    fn draw_text(&mut self, mut origin: Vector2, text: &str, options: TextOptions) {
        if self.align_to_pixel {
            origin = origin.round()
        }

        let draw_list = self.get_draw_list();
        let text = unsafe { ImStr::from_ptr_unchecked(ImString::new(text).as_ptr()) };

        let font = *self.font_ids.get(&options.font).unwrap();
        let _font_size = options.font_size.unwrap_or(0.0);

        let font_token = self.ui.push_font(font);

        let x = match options.centered_horizontal {
            false => origin.x,
            true => origin.x - (self.ui.calc_text_size(text, false, 0.0)[0] / 2.0),
        };
        let y = match options.centered_vertical {
            false => origin.y,
            true => origin.y - (self.ui.calc_text_size(text, false, 0.0)[0] / 2.0),
        };

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

    fn draw_circle(&mut self, _origin: Vector2, _radius: f32, _options: CircleOptions) {
        unimplemented!()
    }
}
