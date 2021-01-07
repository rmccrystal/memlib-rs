use std::sync::mpsc::{channel, Sender};

use winapi::_core::ffi::c_void;
use winapi::_core::ptr::{null, null_mut};
use winapi::Interface;
use winapi::shared::d3d9::{D3D_SDK_VERSION, D3DADAPTER_DEFAULT, D3DCREATE_HARDWARE_VERTEXPROCESSING, Direct3DCreate9, IDirect3D9, LPDIRECT3DDEVICE9};
use winapi::shared::d3d9caps::D3DPRESENT_INTERVAL_ONE;
use winapi::shared::d3d9types::{D3DDEVTYPE_HAL, D3DFMT_A8R8G8B8, D3DFMT_D16, D3DMULTISAMPLE_NONE, D3DPRESENT_PARAMETERS, D3DSWAPEFFECT_DISCARD};
use winapi::shared::dxgiformat::DXGI_FORMAT_UNKNOWN;
use winapi::shared::ntdef::TRUE;
use winapi::shared::windef::{HWND, RECT};
use winapi::shared::winerror::FAILED;
use winapi::um::d2d1::{D2D1_FACTORY_TYPE_MULTI_THREADED, D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_PROPERTIES, D2D1CreateFactory, ID2D1Factory, ID2D1HwndRenderTarget};
use winapi::um::dcommon::{D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_PIXEL_FORMAT, D2D1_SIZE_U};
use winapi::um::dwmapi::DwmExtendFrameIntoClientArea;
use winapi::um::dwrite::{DWRITE_FACTORY_TYPE_SHARED, DWriteCreateFactory, IDWriteFactory};
use winapi::um::unknwnbase::IUnknown;
use winapi::um::uxtheme::MARGINS;
use winapi::um::winuser::{DispatchMessageA, FindWindowA, GetClientRect, GetWindowLongA, GWL_EXSTYLE, HWND_TOPMOST, MSG, PeekMessageA, PM_REMOVE, SetLayeredWindowAttributes, SetWindowLongPtrA, SetWindowPos, ShowWindow, SW_SHOW, SWP_NOMOVE, SWP_NOSIZE, TranslateMessage, WS_EX_TRANSPARENT};

use crate::memory;

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

pub unsafe fn init_d2d(window: HWND) -> memory::Result<(&'static ID2D1HwndRenderTarget, &'static IDWriteFactory)> {
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

pub unsafe fn hijack_window(class_name: &str, window_name: &str) -> memory::Result<HWND> {
    let window = FindWindowA(c_string!(class_name), c_string!(window_name));
    if window.is_null() {
        return Err("Could not find NVIDIA GeForce Overlay window".into());
    }

    // Get the window extended window style
    let style = GetWindowLongA(window, GWL_EXSTYLE);

    // Set the window style to transparent
    SetWindowLongPtrA(window, GWL_EXSTYLE, (style | WS_EX_TRANSPARENT as i32) as _);

    DwmExtendFrameIntoClientArea(window, &MARGINS {
        cxLeftWidth: -1,
        cxRightWidth: -1,
        cyBottomHeight: -1,
        cyTopHeight: -1,
    });

    SetLayeredWindowAttributes(window, 0, 0xFF, 0x02);

    // Set window as topmost
    SetWindowPos(window, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE);

    ShowWindow(window, SW_SHOW);

    Ok(window)
}

// TODO: There is a memory leak here
pub unsafe fn create_d3d_device(window: HWND) -> memory::Result<LPDIRECT3DDEVICE9> {
    let mut params = D3DPRESENT_PARAMETERS {
        Windowed: 1,
        SwapEffect: D3DSWAPEFFECT_DISCARD,
        EnableAutoDepthStencil: 1,
        AutoDepthStencilFormat: D3DFMT_D16,
        PresentationInterval: D3DPRESENT_INTERVAL_ONE,
        hDeviceWindow: window,
        MultiSampleQuality: D3DMULTISAMPLE_NONE,
        BackBufferCount: 1,
        BackBufferFormat: D3DFMT_A8R8G8B8,
        BackBufferWidth: 0,
        BackBufferHeight: 0,
        ..std::mem::zeroed()
    };

    let mut device: LPDIRECT3DDEVICE9 = null_mut();

    let d3d = {
        let device = Direct3DCreate9(D3D_SDK_VERSION);
        if device.is_null() {
            return Err("direct3d device was null".into());
        }
        device.as_mut().unwrap()
    };
    let result = d3d.CreateDevice(
        D3DADAPTER_DEFAULT,
        D3DDEVTYPE_HAL,
        window,
        D3DCREATE_HARDWARE_VERTEXPROCESSING,
        &mut params,
        &mut device,
    );
    if FAILED(result) {
        return Err(format!("Could not create Direct3d device: {:X}", result).into());
    }

    Ok(device)
}

pub unsafe fn create_dx11_device(window: HWND) -> memory::Result<()> {
    D3D11CreateDevice()
}