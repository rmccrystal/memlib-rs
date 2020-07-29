#![cfg(windows)]

use winapi::um::*;
use std::mem;

// Gets a down or up state of a certain key using a VK key code:
// https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
pub fn get_key_state(key: i32) -> bool {
    unsafe { winuser::GetAsyncKeyState(key) != 0 }
}

pub fn move_mouse_relative(dx: i32, dy: i32) {
    let mut input_union: winuser::INPUT_u = unsafe { std::mem::zeroed() };
    unsafe {
        input_union.mi_mut().dx = dx;
        input_union.mi_mut().dy = dy;
        input_union.mi_mut().dwFlags = winuser::MOUSEEVENTF_MOVE;
    }

    let mut input = winuser::INPUT {
        type_: winuser::INPUT_KEYBOARD,
        u: input_union,
    };
    unsafe {
        winuser::SendInput(1, &mut input, mem::size_of::<winuser::INPUT>() as i32);
    }
    let error = unsafe { errhandlingapi::GetLastError() };
    if error != 0 {
        error!("move_mouse_relative failed with error code 0x{:X}", error)
    }
}