use anyhow::Result;
use core::result::Result::Ok;
use winapi::shared::windef::HWND;
use winapi::um::dwmapi::DwmExtendFrameIntoClientArea;
use winapi::um::uxtheme::MARGINS;
use winapi::um::winuser::{FindWindowA, GetWindowLongA, GWL_EXSTYLE, HWND_TOPMOST, SetLayeredWindowAttributes, SetWindowLongPtrA, SetWindowPos, ShowWindow, SW_SHOW, SWP_NOMOVE, SWP_NOSIZE, WS_EX_TRANSPARENT};
use anyhow::*;

pub struct Window {
    pub(crate) hwnd: HWND,
}

impl Window {
    /// Hijacks a window from its class name and window name
    pub fn hijack(class_name: &str, window_name: &str) -> Result<Window> {
        unsafe {
            let hwnd = FindWindowA(c_string!(class_name), c_string!(window_name));
            if hwnd.is_null() {
                bail!(
            "Could not find window with class name {} and window name {}",
            class_name,
            window_name
        );
            }

            // Get the window extended window style
            let style = GetWindowLongA(hwnd, GWL_EXSTYLE);

            // Set the window style to transparent
            SetWindowLongPtrA(hwnd, GWL_EXSTYLE, (style | WS_EX_TRANSPARENT as i32) as _);

            DwmExtendFrameIntoClientArea(
                hwnd,
                &MARGINS {
                    cxLeftWidth: -1,
                    cxRightWidth: -1,
                    cyBottomHeight: -1,
                    cyTopHeight: -1,
                },
            );

            SetLayeredWindowAttributes(hwnd, 0, 0xFF, 0x02);

            // Set window as topmost
            SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE);

            ShowWindow(hwnd, SW_SHOW);

            Ok(Window { hwnd })
        }
    }

    /// Hijacks the NVIDIA overlay
    pub fn hijack_nvidia() -> Result<Window> {
        Self::hijack("CEF-OSC-WIDGET", "NVIDIA GeForce Overlay")
    }
}