use crate::memory::handle_interfaces::winapi_handle::error_code_to_message;
use anyhow::*;
use std::ptr::null_mut;
use winapi::_core::ops::{Deref, DerefMut};
use winapi::shared::d3d9::{
    Direct3DCreate9, IDirect3DDevice9, D3DADAPTER_DEFAULT, D3DCREATE_HARDWARE_VERTEXPROCESSING,
    D3D_SDK_VERSION, LPDIRECT3DDEVICE9,
};
use winapi::shared::d3d9caps::D3DPRESENT_INTERVAL_ONE;
use winapi::shared::d3d9types::{
    D3DDEVTYPE_HAL, D3DFMT_A8R8G8B8, D3DFMT_D16, D3DMULTISAMPLE_NONE, D3DPRESENT_PARAMETERS,
    D3DSWAPEFFECT_DISCARD,
};
use winapi::shared::ntdef::NTSTATUS;
use winapi::shared::windef::HWND;
use winapi::shared::winerror::FAILED;
use winapi::um::dwmapi::DwmExtendFrameIntoClientArea;
use winapi::um::uxtheme::MARGINS;
use winapi::um::winuser::{
    FindWindowA, GetWindowLongA, SetLayeredWindowAttributes, SetWindowLongPtrA, SetWindowPos,
    ShowWindow, GWL_EXSTYLE, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SW_SHOW, WS_EX_TRANSPARENT,
};

macro_rules! c_string {
    ($str:expr) => {
        std::ffi::CString::new($str).unwrap().as_ptr()
    };
}

macro_rules! c_string_w {
    ($str:expr) => {{
        let ptr: *const u16 = {
            let text: Vec<u16> = OsStr::new($str)
                .encode_wide()
                .chain(Some(0).into_iter())
                .collect();
            text.as_ptr()
        };
        ptr
    }};
}

pub(crate) trait ToError {
    fn to_err(self) -> Result<()>;
}

impl ToError for NTSTATUS {
    fn to_err(self) -> Result<()> {
        if FAILED(self) {
            Err(anyhow!(
                "{} ({:X})",
                error_code_to_message(self as _).unwrap_or_default(),
                self
            ))
        } else {
            Ok(())
        }
    }
}

pub unsafe fn hijack_window(class_name: &str, window_name: &str) -> Result<HWND> {
    let window = FindWindowA(c_string!(class_name), c_string!(window_name));
    if window.is_null() {
        bail!(
            "Could not find window with class name {} and window name {}",
            class_name,
            window_name
        );
    }

    // Get the window extended window style
    let style = GetWindowLongA(window, GWL_EXSTYLE);

    // Set the window style to transparent
    SetWindowLongPtrA(window, GWL_EXSTYLE, (style | WS_EX_TRANSPARENT as i32) as _);

    DwmExtendFrameIntoClientArea(
        window,
        &MARGINS {
            cxLeftWidth: -1,
            cxRightWidth: -1,
            cyBottomHeight: -1,
            cyTopHeight: -1,
        },
    );

    SetLayeredWindowAttributes(window, 0, 0xFF, 0x02);

    // Set window as topmost
    SetWindowPos(window, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE);

    ShowWindow(window, SW_SHOW);

    Ok(window)
}

pub struct D3DDevice9(pub(crate) LPDIRECT3DDEVICE9);

impl Drop for D3DDevice9 {
    fn drop(&mut self) {
        unsafe { (*self.0).Release() };
    }
}

impl Deref for D3DDevice9 {
    type Target = IDirect3DDevice9;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref().unwrap() }
    }
}

impl DerefMut for D3DDevice9 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut().unwrap() }
    }
}

pub unsafe fn create_d3d_device(window: HWND) -> anyhow::Result<D3DDevice9> {
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
            bail!("direct3d device was null");
        }
        device.as_mut().unwrap()
    };
    let result = d3d
        .CreateDevice(
            D3DADAPTER_DEFAULT,
            D3DDEVTYPE_HAL,
            window,
            D3DCREATE_HARDWARE_VERTEXPROCESSING,
            &mut params,
            &mut device,
        )
        .to_err()
        .context("Failed to create D3D device")?;

    Ok(D3DDevice9(device))
}
