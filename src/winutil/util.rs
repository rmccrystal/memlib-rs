use std::collections::HashMap;

use winapi::um::winuser::{GetAsyncKeyState, GetKeyboardState, GetKeyState, EnumWindows, GetWindowTextA};
pub use winapi::shared::windef::HWND;
use std::ffi::c_void;
use core::mem;
use winapi::shared::minwindef::{LPARAM, FALSE, BOOL};
use winapi::shared::ntdef::TRUE;

macro_rules! c_string {
    ($str:expr) => {
        std::ffi::CString::new($str).unwrap().as_ptr()
    };
}

macro_rules! c_string_w {
    ($str:expr) => {{
        use std::os::windows::ffi::OsStrExt;
        let _out: Vec<_> = std::ffi::OsStr::new($str)
                .encode_wide()
                .chain(Some(0).into_iter())
                .collect();
        _out
    }};
}

/// Returns a list of all keys and weather or not they are down
pub(crate) fn get_keyboard_state() -> HashMap<i32, bool> {
    /*
    let mut keys = vec![0u8; 256];
    let _ = unsafe { GetKeyState(0) };
    let success = unsafe { GetKeyboardState(keys.as_mut_ptr()) };
    if success == 0 {
        panic!(std::io::Error::last_os_error());
    }

    keys.into_iter()
        .enumerate()
        .map(|state| (state.0 as i32, state.1 & 1 == 1))
        .collect()
     */
    const NUM_KEYS: usize = 254;

    let mut keys = HashMap::with_capacity(NUM_KEYS);
    for i in 0i32..=(NUM_KEYS as i32) {
        keys.insert(i, is_key_down(i));
    }

    keys
}

pub fn is_key_down(key: i32) -> bool {
    let state = unsafe { GetAsyncKeyState(key as _) } as u16;
    (state & 0x8000) != 0
}

pub fn enumerate_window_titles(mut callback: impl FnMut(&[u8], HWND) -> bool) {
    enumerate_windows(|hwnd| {
        let mut str: Vec<u8> = vec![0; 256];
        unsafe { GetWindowTextA(hwnd, str.as_mut_ptr() as _, 256) };

        callback(&str, hwnd)
    })
}

pub fn enumerate_windows<F>(mut callback: F)
    where F: FnMut(HWND) -> bool
{
    let mut trait_obj: &mut dyn FnMut(HWND) -> bool = &mut callback;
    let closure_pointer_pointer: *mut c_void = unsafe { mem::transmute(&mut trait_obj) };

    let lparam = closure_pointer_pointer as LPARAM;
    unsafe { EnumWindows(Some(enumerate_callback), lparam) };
}

unsafe extern "system" fn enumerate_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let closure: &mut &mut dyn FnMut(HWND) -> bool = mem::transmute(lparam as *mut c_void);
    if closure(hwnd) { 1 } else { 0 }
}
