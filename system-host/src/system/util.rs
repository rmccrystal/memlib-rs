#![cfg(windows)]
#![allow(clippy::missing_safety_doc)]

use enigo::MouseControllable;
use log::*;
use winapi::um::*;

/// Gets a down or up state of a certain key using a VK key code:
/// https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
pub fn get_key_state(key: i32) -> bool {
    debug!("get_key_state({})", key);
    unsafe { winuser::GetAsyncKeyState(key) != 0 }
}

pub fn move_mouse_relative(dx: i32, dy: i32) {
    let mut enigo = enigo::Enigo::new();
    enigo.mouse_move_relative(dx, dy);
}
