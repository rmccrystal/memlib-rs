use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::OsStrExt;

use winapi::_core::ptr::{null, null_mut};
use winapi::ctypes::c_void;
use winapi::Interface;
use winapi::shared::dxgiformat::DXGI_FORMAT_UNKNOWN;
use winapi::shared::windef::{HWND, RECT};
use winapi::shared::winerror::FAILED;
use winapi::um::d2d1::{D2D1_FACTORY_TYPE_MULTI_THREADED, D2D1_FEATURE_LEVEL_DEFAULT, D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_USAGE_NONE, D2D1CreateFactory, ID2D1Factory, ID2D1HwndRenderTarget, ID2D1Brush, D2D1_COLOR_F, D2D1_BRUSH_PROPERTIES, ID2D1SolidColorBrush, ID2D1StrokeStyle, ID2D1BrushVtbl, ID2D1RenderTarget};
use winapi::um::dcommon::{D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_PIXEL_FORMAT, D2D1_SIZE_U, D2D1_RECT_F};
use winapi::um::dwmapi::DwmExtendFrameIntoClientArea;
use winapi::um::dwrite::{DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_REGULAR, DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat};
use winapi::um::unknwnbase::IUnknown;
use winapi::um::uxtheme::MARGINS;
use winapi::um::winuser::{FindWindowA, GetClientRect, GetWindowLongA, GWL_EXSTYLE, HWND_TOPMOST, SetLayeredWindowAttributes, SetWindowLongPtrA, SetWindowPos, ShowWindow, SW_SHOW, SWP_NOMOVE, SWP_NOSIZE, WS_EX_TRANSPARENT};

use crate::math::Vector2;
use crate::memory::Result;
use crate::overlay::{BoxOptions, CircleOptions, LineOptions, OverlayInterface, TextOptions, Color};
use winapi::_core::ops::Deref;

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
    pub render: &'static mut ID2D1HwndRenderTarget,
}

impl NvidiaOverlay {
    pub fn init() -> Result<Self> {
        unsafe {
            let window = Self::init_window()?;
            let render = Self::init_d2d(window)?;

            Ok(Self{render, window})
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

    unsafe fn init_d2d(window: HWND) -> Result<&'static mut ID2D1HwndRenderTarget> {
        let mut d2d_factory: *mut c_void = null_mut();
        let result = D2D1CreateFactory(
            D2D1_FACTORY_TYPE_MULTI_THREADED,
            &ID2D1Factory::uuidof(),
            null(),
            &mut d2d_factory
        );
        if FAILED(result) {
            return Err(format!("Could not create D2D1 factory: {:X}", result).into());
        }
        let d2d_factory = (d2d_factory as *mut ID2D1Factory).as_mut().unwrap();

        let mut write_factory: *mut IUnknown = null_mut();
        let result = DWriteCreateFactory(
            DWRITE_FACTORY_TYPE_SHARED,
            &IDWriteFactory::uuidof(),
            &mut write_factory
        );
        if FAILED(result) {
            return Err(format!("Could not create D2D1 write factory: {:X}", result).into());
        }
        let write_factory = (write_factory as *mut IDWriteFactory).as_mut().unwrap();

        let mut text_format: *mut IDWriteTextFormat = null_mut();
        write_factory.CreateTextFormat(
            c_string_w!("Consolas"),
            null_mut(),
            DWRITE_FONT_WEIGHT_REGULAR,
            DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            13.0,
            c_string_w!("en-us"),
            &mut text_format,
        );

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

        Ok(render.as_mut().unwrap())
    }

    fn create_brush(&mut self, color: Color) -> &'static mut ID2D1Brush {
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

            // TODO: There has to be a better way to do this
            #[allow(mutable_transmutes)]
            #[allow(clippy::transmute_ptr_to_ptr)]
            std::mem::transmute((*brush).deref())
        }
    }
}

impl OverlayInterface for NvidiaOverlay {
    fn begin(&mut self) {
        unsafe { self.render.BeginDraw() };
    }

    fn end(&mut self) {
        unsafe { self.render.EndDraw(null_mut(), null_mut()) };
    }

    fn draw_line(&mut self, p1: Vector2, p2: Vector2, options: LineOptions) {
        unimplemented!()
    }

    fn draw_box(&mut self, p1: Vector2, p2: Vector2, options: BoxOptions) {
        unsafe {
            let brush = self.create_brush(options.color.into());
            self.render.DrawRectangle(
                &D2D1_RECT_F {
                    left: p1.x,
                    right: p2.x,
                    top: p1.y,
                    bottom: p2.y,
                },
                brush as _,
                options.width,
                null_mut()
            );
        }
    }

    fn draw_text(&mut self, origin: Vector2, text: &str, options: TextOptions) {
        unimplemented!()
    }

    fn draw_circle(&mut self, origin: Vector2, radius: f32, options: CircleOptions) {
        unimplemented!()
    }
}