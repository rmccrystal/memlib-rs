use winapi::shared::windef::HWND;
use winapi::um::processthreadsapi::GetCurrentProcessId;
use crate::winutil::find_window;
use winapi::um::winuser::GetWindowThreadProcessId;
use winapi::um::winbase::IsBadReadPtr;

pub fn get_window() -> HWND {
    unsafe {
        let current_pid = GetCurrentProcessId();
        find_window(|hwnd| {
            let mut pid = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            pid == current_pid
        }).expect("Could not find process PID")
    }
}

pub fn is_valid_address<T>(addr: *const T) -> bool {
    unsafe { IsBadReadPtr(addr as _, 4) == 0 }
}