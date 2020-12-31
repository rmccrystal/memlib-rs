use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::OsStrExt;

use winapi::_core::ptr::{null, null_mut};
use winapi::ctypes::c_void;
use winapi::Interface;
use winapi::shared::dxgiformat::DXGI_FORMAT_UNKNOWN;
use winapi::shared::windef::{HWND, RECT};
use winapi::shared::winerror::FAILED;
use winapi::um::d2d1::{D2D1_FACTORY_TYPE_MULTI_THREADED, D2D1_FEATURE_LEVEL_DEFAULT, D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_USAGE_NONE, D2D1CreateFactory, ID2D1Factory, ID2D1HwndRenderTarget, ID2D1Brush, D2D1_COLOR_F, D2D1_BRUSH_PROPERTIES, ID2D1SolidColorBrush, ID2D1StrokeStyle, ID2D1BrushVtbl, ID2D1RenderTarget, D2D1_ANTIALIAS_MODE_ALIASED, D2D1_ROUNDED_RECT, D2D1_POINT_2F, D2D1_DRAW_TEXT_OPTIONS_NONE};
use winapi::um::dcommon::{D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_PIXEL_FORMAT, D2D1_SIZE_U, D2D1_RECT_F, DWRITE_MEASURING_MODE_NATURAL};
use winapi::um::dwmapi::DwmExtendFrameIntoClientArea;
use winapi::um::dwrite::{DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_REGULAR, DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat};
use winapi::um::unknwnbase::IUnknown;
use winapi::um::uxtheme::MARGINS;
use winapi::um::winuser::{FindWindowA, GetClientRect, GetWindowLongA, GWL_EXSTYLE, HWND_TOPMOST, SetLayeredWindowAttributes, SetWindowLongPtrA, SetWindowPos, ShowWindow, SW_SHOW, SWP_NOMOVE, SWP_NOSIZE, WS_EX_TRANSPARENT};

use crate::math::Vector2;
use crate::memory::Result;
use crate::overlay::{BoxOptions, CircleOptions, LineOptions, OverlayInterface, TextOptions, Color, Font};
use cached::proc_macro::cached;

macro_rules! c_string {
    ($str:expr) => {std::ffi::CString::new($str).unwrap().as_ptr()};
}

macro_rules! c_string_w {
    ($str:expr) => {{
        let ptr: *const u16 = {
            let text: Vec<u16> = OsStr::new($str).encode_wide(). chain(Some(0).into_iter()).collect();
            text.as_ptr()
        };
        ptr
    }}
}

pub struct NvidiaOverlay {
    window: HWND,
    render: &'static ID2D1RenderTarget,
    write_factory: &'static IDWriteFactory,
}

impl NvidiaOverlay {
    pub fn init() -> Result<Self> {
        unsafe {
            let window = Self::init_window()?;
            let (render, write_factory) = Self::init_d2d(window)?;

            let render: &'static ID2D1RenderTarget = &*render;

            render.SetAntialiasMode(D2D1_ANTIALIAS_MODE_ALIASED);

            Ok(Self { render, window, write_factory })
        }
    }

    unsafe fn init_window() -> Result<HWND> {
        let window = FindWindowA(c_string!("CEF-OSC-WIDGET"), c_string!("NVIDIA GeForce Overlay"));
        if window.is_null() {
            return Err("Could not find NVIDIA GeForce Overlay window".into());
        }

        // Get the window extended window style
        let style = GetWindowLongA(window, GWL_EXSTYLE);
        // Set the window style to transparent
        // TODO: Shouldn't this already be transparent?
        SetWindowLongPtrA(window, GWL_EXSTYLE, (style | WS_EX_TRANSPARENT as i32) as _);

        // Set the window transparency
        DwmExtendFrameIntoClientArea(window, &MARGINS {
            cxLeftWidth: -1,
            cxRightWidth: -1,
            cyBottomHeight: -1,
            cyTopHeight: -1,
        });

        SetLayeredWindowAttributes(window, 0, 0xFF, 0x02);

        // Set window as topmost
        // TODO: I don't think we should need this either because nvidia is already topmost
        SetWindowPos(window, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE);

        ShowWindow(window, SW_SHOW);

        Ok(window)
    }

    unsafe fn init_d2d(window: HWND) -> Result<(&'static ID2D1HwndRenderTarget, &'static IDWriteFactory)> {
        let mut d2d_factory: *mut c_void = null_mut();
        let result = D2D1CreateFactory(
            D2D1_FACTORY_TYPE_MULTI_THREADED,
            &ID2D1Factory::uuidof(),
            null(),
            &mut d2d_factory,
        );
        if FAILED(result) {
            return Err(format!("Could not create D2D1 factory: {:X}", result).into());
        }
        let d2d_factory = (d2d_factory as *mut ID2D1Factory).as_mut().unwrap();

        let mut write_factory: *mut IUnknown = null_mut();
        let result = DWriteCreateFactory(
            DWRITE_FACTORY_TYPE_SHARED,
            &IDWriteFactory::uuidof(),
            &mut write_factory,
        );
        if FAILED(result) {
            return Err(format!("Could not create D2D1 write factory: {:X}", result).into());
        }
        let write_factory = (write_factory as *mut IDWriteFactory).as_mut().unwrap();

        Self::init_fonts(write_factory);

        let mut rect: RECT = std::mem::zeroed();

        GetClientRect(window, &mut rect as _);

        // Create render target
        let mut render: *mut ID2D1HwndRenderTarget = null_mut();
        let result = d2d_factory.CreateHwndRenderTarget(
            &D2D1_RENDER_TARGET_PROPERTIES {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_UNKNOWN,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                ..Default::default()
            },
            &D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd: window,
                pixelSize: D2D1_SIZE_U { height: (rect.bottom - rect.top) as _, width: (rect.right - rect.left) as _ },
                ..Default::default()
            },
            &mut render as _,
        );
        if FAILED(result) {
            return Err(format!("Could not create HWND render target: {:X}", result).into());
        }

        Ok((render.as_mut().unwrap(), write_factory))
    }

    unsafe fn init_fonts(write_factory: &mut IDWriteFactory) {
        // TODO
    }

    fn create_brush(&mut self, color: Color) -> *mut ID2D1Brush {
        unsafe {
            let mut brush: *mut ID2D1SolidColorBrush = null_mut();
            let (r, g, b, a) = color.to_rgba();
            let result = self.render.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: (r as f32) / 255.0,
                    g: (g as f32) / 255.0,
                    b: (b as f32) / 255.0,
                    a: (a as f32) / 255.0,
                },
                null_mut(),
                &mut brush as _);
            if FAILED(result) {
                panic!("Could not create D2D1 factory: {:X}", result);
            }

            brush as _
        }
    }

    pub fn create_text_format(&self, font: Font, size: f32) -> *mut IDWriteTextFormat {
        _create_text_format(self.write_factory as *const _ as _, font, size as _) as _
    }
}

/// Please do not use this function outside of the wrapper method
#[cached]
fn _create_text_format(write_factory: usize, font: Font, size: i32) -> usize /* *mut IDWriteTextFormat */ {
    unsafe {
        let write_factory: *const IDWriteFactory = write_factory as _;
        // TODO: use in memory fonts
        let font_name = match font {
            Font::Default => "Times new Roman",
            Font::Pixel => "Small Fonts",
            Font::Tahoma => "Tahoma",
            Font::Verdana => "Verdana",
        };

        let mut text_format: *mut IDWriteTextFormat = null_mut();
        (*write_factory).CreateTextFormat(
            c_string_w!(font_name),
            null_mut(),
            DWRITE_FONT_WEIGHT_REGULAR,
            DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            size as f32,
            c_string_w!("en-us"),
            &mut text_format,
        );

        text_format as _
    }
}

// TODO: Clean everything up upon drop

impl OverlayInterface for NvidiaOverlay {
    fn begin(&mut self) {
        unsafe {
            self.render.BeginDraw();
            self.render.Clear(null());
        };
    }

    fn end(&mut self) {
        unsafe {
            self.render.EndDraw(null_mut(), null_mut())
        };
    }

    fn draw_line(&mut self, p1: Vector2, p2: Vector2, options: LineOptions) {
        unsafe {
            let brush = self.create_brush(options.color.into());

            let p1 = D2D1_POINT_2F { x: p1.x, y: p1.y };
            let p2 = D2D1_POINT_2F { x: p2.x, y: p2.y };

            self.render.DrawLine(p1, p2, brush, options.width, null_mut());
        }
    }

    fn draw_box(&mut self, p1: Vector2, p2: Vector2, options: BoxOptions) {
        unsafe {
            let brush = self.create_brush(options.color.into());
            let rect = D2D1_RECT_F {
                left: p1.x,
                right: p2.x,
                top: p1.y,
                bottom: p2.y,
            };

            let (filled, rounded) = (options.filled, options.rounding > 0.0);

            match (filled, rounded) {
                (false, false) => self.render.DrawRectangle(
                    &rect,
                    brush,
                    options.width,
                    null_mut(),
                ),
                (true, false) => self.render.FillRectangle(&rect, brush),
                // any rectangle that is rounded
                (filled, true) => {
                    let rounded_rect = D2D1_ROUNDED_RECT {
                        rect,
                        radiusX: options.rounding,
                        radiusY: options.rounding,
                    };
                    match filled {
                        true => self.render.FillRoundedRectangle(&rounded_rect, brush),
                        false => self.render.DrawRoundedRectangle(
                            &rounded_rect,
                            brush,
                            options.width,
                            null_mut()),
                    }
                }
            }
        }
    }

    fn draw_text(&mut self, origin: Vector2, text: &str, options: TextOptions) {
        unsafe {
            // TODO: we should probably cache this or time it to see if it makes any difference
            let format = self.create_text_format(options.font, options.font_size.unwrap_or(12.0));
            let brush = self.create_brush(options.color.into());
            self.render.DrawText(
                c_string_w!(text),
                text.len() as _,
                format,
                &D2D1_RECT_F { left: origin.x, top: origin.y, right: 1000.0, bottom: 800.0 }, // TODO
                brush,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            );
        }
    }

    fn draw_circle(&mut self, origin: Vector2, radius: f32, options: CircleOptions) {
        todo!()
        // https://stackoverflow.com/questions/13854168/how-to-draw-a-circle-with-id2d1pathgeometry
    }
}