#![cfg(windows)]

use winapi::um::*;
use log::*;
use std::mem;

// Gets a down or up state of a certain key using a VK key code:
// https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
pub unsafe fn get_key_state(key: i32) -> bool {
    debug!("get_key_state({})", key);
    winuser::GetAsyncKeyState(key) != 0
}

pub unsafe fn move_mouse_relative(dx: i32, dy: i32) {
    debug!("move_mouse_relative({}, {})", dx, dy);
    let mut input_union: winuser::INPUT_u = std::mem::zeroed();
    input_union.mi_mut().dx = dx;
    input_union.mi_mut().dy = dy;
    input_union.mi_mut().dwFlags = winuser::MOUSEEVENTF_MOVE;

    let mut input = winuser::INPUT {
        type_: winuser::INPUT_KEYBOARD,
        u: winapi::de,
    };

    winuser::SendInput(1, &mut input, mem::size_of::<winuser::INPUT>() as i32);
    let error = errhandlingapi::GetLastError();
    if error != 0 {
        error!("move_mouse_relative failed with error code 0x{:X}", error)
    }
}