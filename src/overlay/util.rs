use std::ptr::null_mut;

use anyhow::*;
use winapi::_core::ops::{Deref, DerefMut};
use winapi::shared::d3d9::{
    D3D_SDK_VERSION, D3DADAPTER_DEFAULT, D3DCREATE_HARDWARE_VERTEXPROCESSING, Direct3DCreate9,
    IDirect3DDevice9, LPDIRECT3DDEVICE9,
};
use winapi::shared::d3d9caps::D3DPRESENT_INTERVAL_ONE;
use winapi::shared::d3d9types::{
    D3DDEVTYPE_HAL, D3DFMT_A8R8G8B8, D3DFMT_D16, D3DMULTISAMPLE_NONE, D3DPRESENT_PARAMETERS,
    D3DSWAPEFFECT_DISCARD,
};
use winapi::um::winuser::*;
use crate::winutil::ToError;

use crate::overlay::window::Window;

macro_rules! c_string {
    ($str:expr) => {
        std::ffi::CString::new($str).unwrap().as_ptr()
    };
}

#[allow(unused_macros)]
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

pub fn is_key_down(key: i32) -> bool {
    unsafe { (GetAsyncKeyState(key)) != 0 }
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

pub unsafe fn create_d3d_device(window: &Window) -> anyhow::Result<D3DDevice9> {
    let window = window.hwnd;
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
    d3d.CreateDevice(
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
