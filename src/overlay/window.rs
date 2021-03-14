use core::result::Result::Ok;
use std::io;

use anyhow::*;
use anyhow::Result;
use log::*;
use winapi::_core::ptr::null_mut;
use winapi::shared::windef::{HWND, RECT};
use winapi::um::dwmapi::DwmExtendFrameIntoClientArea;
use winapi::um::libloaderapi::{GetProcAddress, LoadLibraryA, FreeLibrary};
use winapi::um::processthreadsapi::GetCurrentProcessId;
use winapi::um::uxtheme::MARGINS;
use winapi::um::winuser::*;

use crate::winutil::{inject_func};
use std::ptr::null;
use std::time::{Instant, Duration};


pub struct Window {
    pub(crate) hwnd: HWND,
    pub target_hwnd: Option<HWND>,
    last_update: Instant,
    last_clickthrough: bool,
    last_foreground_window: Option<HWND>,
}

unsafe impl Send for Window {}

unsafe impl Sync for Window {}

impl Window {
    pub fn from_hwnd(hwnd: HWND) -> Result<Self> {
        let mut window = Self {
            hwnd,
            target_hwnd: None,
            last_update: Instant::now() - Duration::from_secs(60),
            last_clickthrough: true,
            last_foreground_window: None,
        };

        window.set_style(GWL_EXSTYLE, WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_TOOLWINDOW)?;
        window.extend_into_client_area();
        window.set_alpha(0xFF);
        window.show();

        window.set_clickthrough(true);

        Ok(window)
    }

    /// Hijacks a window from its class name and window name
    pub fn hijack(class_name: &str, window_name: &str) -> Result<Self> {
        let hwnd = find_window(Some(class_name), window_name)
            .ok_or_else(|| anyhow!(
                "Could not find window with class name {} and window name {}",
                class_name,
                window_name
                ))?;

        trace!("Found hWnd with class_name: {} and window_name: {}: {:p}", class_name, window_name, hwnd);

        Self::from_hwnd(hwnd)
    }

    pub fn create() -> Result<Self> {
        let class_name = "Edit";
        let window_name = "Notepad";

        let hwnd = unsafe {
            CreateWindowExA(
                WS_EX_LAYERED | WS_EX_TRANSPARENT,
                c_string!(class_name),
                c_string!(window_name),
                WS_POPUP,
                0,  // TODO
                0,
                100,
                200,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
            )
        };

        Self::from_hwnd(hwnd)
    }

    pub fn target_window(&mut self, window_name: &str) -> Result<()> {
        let window = find_window(None, window_name)
            .ok_or_else(|| anyhow!("Could not find window {}", window_name))?;

        self.target_hwnd = Some(window);

        self.update_target_window();

        Ok(())
    }

    pub fn tick(&mut self) {
        self.handle_messages();

        if self.last_update < (Instant::now() - Duration::from_secs(1)) {
            self.update_target_window();
            self.set_above_foreground_window();

            self.last_update = Instant::now();
        }
    }

    fn update_target_window(&self) {
        if self.target_hwnd.is_none() {
            return;
        }

        let target_hwnd = self.target_hwnd.unwrap();
        unsafe {
            let mut rect = std::mem::zeroed();
            GetWindowRect(target_hwnd, &mut rect);

            if rect.bottom == 0 && rect.top == 0 {
                panic!("Error updating target window: could not get the window rect");
            }

            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

            trace!("Moving window to X: {}, Y: {}, width: {}, height: {}", rect.left, rect.top, width, height);
            MoveWindow(self.hwnd, rect.left, rect.top, width, height, 1);
        }
    }

    /// Hijacks the NVIDIA overlay
    pub fn hijack_nvidia() -> Result<Window> {
        Self::hijack("CEF-OSC-WIDGET", "NVIDIA GeForce Overlay")
    }

    fn set_topmost(&self) {
        unsafe { SetWindowPos(self.hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE) };
    }

    pub fn bypass_screenshots(&self, enabled: bool) -> Result<()> {
        self.set_affinity(if enabled { WindowAffinity::WdaExcludeFromCapture } else { WindowAffinity::WdaNone })
    }

    pub fn set_alpha(&self, alpha: u8) {
        unsafe { SetLayeredWindowAttributes(self.hwnd, 0, alpha, 0x02); }
    }

    /// Runs DwmExtendFrameIntoClientArea with negative margins
    pub fn extend_into_client_area(&self) {
        unsafe {
            DwmExtendFrameIntoClientArea(
                self.hwnd,
                &MARGINS {
                    cxLeftWidth: -1,
                    cxRightWidth: -1,
                    cyBottomHeight: -1,
                    cyTopHeight: -1,
                },
            );
        }
    }

    /// Sets the window position to be one layer above the current foreground window
    pub fn set_above_foreground_window(&self) {
        unsafe {
            let window = GetWindow(GetForegroundWindow(), GW_HWNDPREV);
            if window != self.hwnd {
                SetWindowPos(
                    self.hwnd,
                    window,
                    0,
                    0,
                    0,
                    0,
                    SWP_ASYNCWINDOWPOS | SWP_NOMOVE | SWP_NOSIZE,
                );
                UpdateWindow(self.hwnd);
            }
        }
    }

    pub fn show(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_SHOW);
            UpdateWindow(self.hwnd);
        };
    }

    pub fn get_rect(&self) -> RECT {
        let mut rect: RECT = Default::default();
        unsafe { GetClientRect(self.hwnd, &mut rect); }
        rect
    }

    /// Handles the messages of a window. If a message is handled, the funciton returns true
    pub fn handle_messages(&self) -> bool {
        unsafe {
            let mut msg = std::mem::zeroed();
            if PeekMessageA(&mut msg, self.hwnd, 0, 0, PM_REMOVE) > 0 {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
                true
            } else {
                false
            }
        }
    }

    pub fn set_clickthrough(&mut self, clickthrough: bool) {
        if clickthrough != self.last_clickthrough {
            self.last_clickthrough = clickthrough;

            self.set_style_flag(GWL_EXSTYLE, WS_EX_TRANSPARENT, clickthrough).expect("Could not set clickthrough");

            unsafe {
                match clickthrough {
                    false => {
                        self.last_foreground_window = Some(GetForegroundWindow());
                        SetForegroundWindow(self.hwnd);
                    },
                    true => {
                        if let Some(window) = self.last_foreground_window {
                            SetForegroundWindow(window);
                            self.last_foreground_window = None;
                        }
                    }
                }
            };
        }
    }

    /// Sets a single style flag
    pub fn set_style_flag(&self, n_index: i32, flag: u32, enabled: bool) -> Result<()> {
        let style = self.get_style(n_index)?;
        let style = match enabled {
            true => style | flag,
            false => style & !flag,
        };
        self.set_style(n_index, style)?;

        Ok(())
    }

    /// Sets a window's style using SetWindowLongA
    pub fn set_style(&self, n_index: i32, flags: u32) -> Result<()> {
        unsafe {
            if self.get_style(n_index)? != flags {
                let result = SetWindowLongA(self.hwnd, n_index, flags as _);
                if result == 0 {
                    return Err(anyhow!("SetWindowLongA failed: {}", io::Error::last_os_error()));
                }
            }
        }

        Ok(())
    }

    pub fn get_style(&self, n_index: i32) -> Result<u32> {
        let style = unsafe { GetWindowLongA(self.hwnd, n_index) };
        Ok(style as _)
    }

    pub fn push_style(&self, n_index: i32, flags: u32) -> Result<()> {
        unsafe {
            let style = GetWindowLongA(self.hwnd, n_index);
            self.set_style(n_index, style as u32 | flags)
        }
    }

    pub fn set_affinity(&self, affinity: WindowAffinity) -> Result<()> {
        if self.get_affinity()? == affinity {
            return Ok(());
        }
        unsafe {
            // If the HWND is owned by this process, we can just call swda
            if GetCurrentProcessId() == self.get_owner_pid()? {
                let result = SetWindowDisplayAffinity(self.hwnd, affinity as _);
                if result == 0 {
                    Err(anyhow!("SetWindowDisplayAffinity failed: {}", io::Error::last_os_error()))
                } else {
                    Ok(())
                }
            } else { // otherwise, we have to set it remotely
                self.set_remote_affinity(affinity)?;
                let actual_affinity = self.get_affinity()?;
                if affinity != actual_affinity {
                    bail!("Setting remote affinity did not work. affinity: {:?}, actual_affinity: {:?}", affinity, actual_affinity);
                }
                Ok(())
            }
        }
    }

    pub fn get_affinity(&self) -> Result<WindowAffinity> {
        unsafe {
            let mut affinity = WindowAffinity::WdaNone;
            let success = GetWindowDisplayAffinity(self.hwnd, std::mem::transmute(&mut affinity));
            if success == 0 {
                bail!("GetWindowDisplayAffinity failed: {}", std::io::Error::last_os_error())
            }

            Ok(affinity)
        }
    }

    /// Sets the window affinity when the HWND isn't owned by this process (nvidia for example)
    fn set_remote_affinity(&self, affinity: WindowAffinity) -> Result<()> {
        debug!("Remotely setting affinity to {:?}", affinity);

        let pid = self.get_owner_pid()?;

        let user32 = unsafe { LoadLibraryA(c_string!("user32.dll")) };
        let swda = unsafe {
            GetProcAddress(
                user32,
                c_string!("SetWindowDisplayAffinity"),
            )
        };

        #[repr(C)]
        struct Data {
            pub affinity: u32,
            pub hwnd: usize,
            pub swda: extern "stdcall" fn(usize, u32),
            pub handled: bool,
        }
        extern "C" fn injected_func(data: &mut Data) -> u32 {
            (data.swda)(data.hwnd as _, data.affinity);
            data.handled = true;
            1
        }
        let data = Data {
            hwnd: self.hwnd as _,
            affinity: affinity as u32,
            swda: unsafe { std::mem::transmute(swda) },
            handled: false,
        };

        let (status, data) = inject_func(pid, injected_func, &data).unwrap();
        assert_eq!(status, 1);
        assert_eq!(data.handled, true);

        let _ = unsafe { FreeLibrary(user32) };

        Ok(())
    }

    /// Gets the process that owns the HWND
    fn get_owner_pid(&self) -> Result<u32> {
        let mut pid = 0;
        let _ = unsafe { GetWindowThreadProcessId(self.hwnd, &mut pid) };
        if pid == 0 {
            bail!("GetWindowThreadProcessId failed");
        }
        Ok(pid)
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            if GetCurrentProcessId() == self.get_owner_pid().unwrap() {
                DestroyWindow(self.hwnd);
            }
        }
    }
}

fn find_window(class_name: Option<&str>, window_name: &str) -> Option<HWND> {
    unsafe {
        let class_name = class_name.map(|n| std::ffi::CString::new(n).unwrap());
        let hwnd = if class_name.is_none() {
            FindWindowA(
                null(),
                c_string!(window_name),
            )
        } else {
            FindWindowA(
                class_name.unwrap().as_ptr(),
                c_string!(window_name),
            )
        };
        if hwnd.is_null() {
            None
        } else {
            Some(hwnd)
        }
    }
}

// https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowdisplayaffinity
#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum WindowAffinity {
    /// Imposes no restrictions on where the window can be displayed.
    WdaNone = 0x0,

    /// The window content is displayed only on a monitor. Everywhere else, the window appears with no content.
    WdaMonitor = 0x1,

    /// The window is displayed only on a monitor. Everywhere else, the window does not appear at all.
    /// One use for this affinity is for windows that show video recording controls, so that the controls are not included in the capture.
    /// Introduced in Windows 10 Version 2004. See remarks about compatibility regarding previous versions of Windows.
    WdaExcludeFromCapture = 0x11,
}
