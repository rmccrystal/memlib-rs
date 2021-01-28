use anyhow::Result;
use core::result::Result::Ok;
use winapi::shared::windef::{HWND, RECT};
use winapi::um::dwmapi::DwmExtendFrameIntoClientArea;
use winapi::um::uxtheme::MARGINS;
use winapi::um::winuser::*;
use anyhow::*;
use winapi::um::errhandlingapi::GetLastError;
use crate::winutil::{ToError, inject_shellcode};
use winapi::_core::ptr::null_mut;
use winapi::um::libloaderapi::{GetProcAddress, LoadLibraryA};
use winapi::um::processthreadsapi::GetCurrentProcessId;

pub struct Window {
    pub(crate) hwnd: HWND,
}

impl Window {
    /// Hijacks a window from its class name and window name
    pub fn hijack(class_name: &str, window_name: &str) -> Result<Window> {
        unsafe {
            let hwnd = Self::find_window(class_name, window_name)
                .ok_or_else(|| anyhow!(
                "Could not find window with class name {} and window name {}",
                class_name,
                window_name
                ))?;

            let window = Self { hwnd };

            window.push_style(WS_EX_LAYERED | WS_EX_TRANSPARENT)?;
            window.extend_into_client_area();
            window.set_alpha(0xFF);

            // Set window as topmost
            SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE);

            window.set_affinity(WindowAffinity::WdaExcludefromcapture)?;

            window.show();

            Ok(Window { hwnd })
        }
    }

    pub fn create() -> Result<Window> {
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
                1920,
                1080,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
            )
        };

        let window = Self { hwnd };

        window.extend_into_client_area();
        window.show();

        Ok(window)
    }

    /// Hijacks the NVIDIA overlay
    pub fn hijack_nvidia() -> Result<Window> {
        Self::hijack("CEF-OSC-WIDGET", "NVIDIA GeForce Overlay")
    }

    fn find_window(class_name: &str, window_name: &str) -> Option<HWND> {
        unsafe {
            let hwnd = FindWindowA(c_string!(class_name), c_string!(window_name));
            if hwnd.is_null() {
                None
            } else {
                Some(hwnd)
            }
        }
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
            let foreground_window = GetWindow(GetForegroundWindow(), GW_HWNDPREV);
            if foreground_window != self.hwnd {
                SetWindowPos(
                    self.hwnd,
                    foreground_window,
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
            let mut msg = unsafe { std::mem::zeroed() };
            if PeekMessageA(&mut msg, self.hwnd, 0, 0, PM_REMOVE) > 0 {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
                true
            } else {
                false
            }
        }
    }

    /// Sets a window's style using SetWindowLongA
    pub fn set_style(&self, flags: u32) -> Result<()> {
        unsafe {
            if GetWindowLongA(self.hwnd, GWL_STYLE) != flags as i32 {
                let result = SetWindowLongA(self.hwnd, GWL_STYLE, flags as _);
                if result == 0 {
                    return Err(anyhow!("SetWindowLongA failed with code {:X}", GetLastError()));
                }
            }
        }

        Ok(())
    }

    pub fn push_style(&self, flags: u32) -> Result<()> {
        unsafe {
            let style = GetWindowLongA(self.hwnd, GWL_EXSTYLE);
            println!("style = {:X}", style);
            println!("flags = {:X}", style);
            println!("a = {:X}", style);
            dbg!(style as u32 | flags);
            self.set_style(style as u32 | flags)
        }
    }

    pub fn set_affinity(&self, affinity: WindowAffinity) -> Result<()> {
        unsafe {
            // If the HWND is owned by this process, we can just call swda
            if GetCurrentProcessId() == self.get_owner_pid()? {
                let result = SetWindowDisplayAffinity(self.hwnd, affinity as _);
                if result != 0 {
                    Err(anyhow!("SetWindowDisplayAffinity failed with code {:X}", GetLastError()))
                } else {
                    Ok(())
                }
            } else { // otherwise, we have to set it remotely
                self.set_remote_affinity(affinity)
            }
        }
    }

    /// Sets the window affinity when the HWND isn't owned by this process (nvidia for example)
    fn set_remote_affinity(&self, affinity: WindowAffinity) -> Result<()> {
        let pid = self.get_owner_pid()?;

        dbg!(pid);

        let swda = unsafe {
            GetProcAddress(
                LoadLibraryA(c_string!("user32.dll")),
                c_string!("SetWindowDisplayAffinity")
            )
        };
        let swda_bytes = (swda as u64).to_le_bytes();

        let dw_affinity = affinity as u32;
        let affinity_bytes = dw_affinity.to_be_bytes();

        let hwnd_bytes = ((self.hwnd as u32) + 1000).to_be_bytes();

        println!("affinity: {:X}, hwnd: {:X}, swda: {:p}", affinity as u32, self.hwnd as u32, swda);

        let mut shellcode = Vec::new();

        // mov edx, dwAffinity
        shellcode.push(0xBA);
        shellcode.extend_from_slice(&affinity_bytes);

        // mov ecx, hWnd
        shellcode.push(0xB9);
        shellcode.extend_from_slice(&hwnd_bytes);

        // mov eax, SetWindowDisplayAffinity
        shellcode.extend_from_slice(&[0x48, 0xB8]);
        shellcode.extend_from_slice(&swda_bytes);

        // call rax
        shellcode.extend_from_slice(&[0xFF, 0xD0]);

        // retn
        shellcode.push(0xC3);

        let result = unsafe { inject_shellcode(&shellcode, pid)? };
        println!("{:X?}", shellcode);
        dbg!(result);

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

// https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowdisplayaffinity
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum WindowAffinity {
    /// Imposes no restrictions on where the window can be displayed.
    WdaNone = 0x0,

    /// The window content is displayed only on a monitor. Everywhere else, the window appears with no content.
    WdaMonitor = 0x1,

    /// The window is displayed only on a monitor. Everywhere else, the window does not appear at all.
    /// One use for this affinity is for windows that show video recording controls, so that the controls are not included in the capture.
    /// Introduced in Windows 10 Version 2004. See remarks about compatibility regarding previous versions of Windows.
    WdaExcludefromcapture = 0x11,
}
