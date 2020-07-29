#![cfg(windows)]

use winapi::um::*;
use std::mem;

// Gets a down or up state of a certain key using a VK key code:
// https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
pub fn get_key_state(key: i32) -> bool {
    unsafe { winuser::GetAsyncKeyState(key) != 0 }
}

pub fn move_mouse_relative(dx: i32, dy: i32) {
    let mut input = winapi::INPUT {
        type_: winuser::INPUT_KEYBOARD,
        union_: winuser::MOUSEINPUT {
            dx,
            dy,
            mouseData: 0,
            dwFlags: winuser::MOUSEEVENTF_MOVE,
            time: 0,
            dwExtraInfo: ()
        }
    };
    unsafe {
        winuser::SendInput(1, &mut input, mem::size_of::<winapi::INPUT>() as i32);
    }
}