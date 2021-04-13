use std::collections::HashMap;

use winapi::um::winuser::{GetAsyncKeyState, GetKeyboardState, GetKeyState, EnumWindows, GetWindowTextA, GetClassNameA, GetWindowThreadProcessId};
pub use winapi::shared::windef::HWND;
use std::ffi::{c_void, CStr};
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

#[derive(Clone, Debug)]
pub struct Window {
    pub class: String,
    pub class_bytes: Vec<u8>,
    pub title: Option<String>,
    pub title_bytes: Vec<u8>,
    pub hwnd: HWND,
    pub pid: u32,
}

pub fn get_windows() -> Vec<Window> {
    let mut windows = Vec::new();
    enumerate_windows(|hwnd| {
        unsafe {
            let mut buf: Vec<u8> = vec![0; 256];

            let class = GetClassNameA(hwnd, buf.as_mut_ptr() as _, 256);
            let class = u8_to_string(&buf).unwrap();
            let class_bytes = buf.iter().map(|n| *n).take_while(|&n| n != 0).collect::<Vec<_>>();

            GetWindowTextA(hwnd, buf.as_mut_ptr() as _, 256);
            let title = u8_to_string(&buf);
            let title_bytes = buf.iter().map(|n| *n).take_while(|&n| n != 0).collect::<Vec<_>>();

            let mut pid = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);

            windows.push(Window {
                hwnd,
                class,
                class_bytes,
                pid,
                title,
                title_bytes,
            })
        }
        true
    });
    windows
}

fn enumerate_window_titles(mut callback: impl FnMut(&[u8], HWND) -> bool) {
    enumerate_windows(|hwnd| {
        let mut str: Vec<u8> = vec![0; 256];
        unsafe { GetWindowTextA(hwnd, str.as_mut_ptr() as _, 256) };

        callback(&str, hwnd)
    })
}

/// Enumerates all windows and returns the window where the callback returns true
fn find_window(mut callback: impl FnMut(HWND) -> bool) -> Option<HWND> {
    let mut window = None;
    enumerate_windows(|hwnd| {
        // if window found
        if callback(hwnd) {
            window = Some(hwnd);
            false   // stop enumeration
        } else {
            true   // continue
        }
    });

    window
}

fn enumerate_windows<F>(mut callback: F)
    where F: FnMut(HWND) -> bool
{
    let mut trait_obj: &mut dyn FnMut(HWND) -> bool = &mut callback;
    let closure_pointer_pointer: *mut c_void = unsafe { mem::transmute(&mut trait_obj) };

    let lparam = closure_pointer_pointer as LPARAM;
    unsafe { EnumWindows(Some(enumerate_callback), lparam) };
}

// To continue enumeration, the callback function must return TRUE; to stop enumeration, it must return FALSE.
unsafe extern "system" fn enumerate_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let closure: &mut &mut dyn FnMut(HWND) -> bool = mem::transmute(lparam as *mut c_void);
    if closure(hwnd) { 1 } else { 0 }
}

fn u8_to_string(src: &[u8]) -> Option<String> {
    let nul_range_end = src.iter()
        .position(|&c| c == b'\0')
        .unwrap_or(src.len()); // default to length if no `\0` present
    String::from_utf8(src[0..nul_range_end].to_vec()).ok()
}