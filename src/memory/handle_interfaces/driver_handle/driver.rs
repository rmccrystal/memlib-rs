use super::*;
use crate::memory::Result;
use std::ffi::CString;
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};

pub const HOOKED_FN_NAME: &str = "NtQueryCompositionSurfaceStatistics";

impl DriverProcessHandle {
    pub(crate) unsafe fn init_hook() -> crate::memory::Result<()> {
        LoadLibraryA(CString::new("user32.dll")?.as_ptr());
        LoadLibraryA(CString::new("win32u.dll")?.as_ptr());
        if GetModuleHandleA(CString::new("win32u.dll")?.as_ptr()).is_null() {
            return Err("win32u.dll failed to load".into());
        }

        trace!("Loaded user32.dll and win32u.dll");

        Ok(())
    }
    pub(crate) unsafe fn get_hook() -> Result<extern "stdcall" fn(*mut c_void)> {
        let addr = GetProcAddress(
            GetModuleHandleA(CString::new("win32u.dll")?.as_ptr()),
            CString::new(HOOKED_FN_NAME)?.as_ptr(),
        );
        Ok(std::mem::transmute(
            addr.as_mut()
                .ok_or(format!("Could not find {}", HOOKED_FN_NAME))?,
        ))
    }

    pub(crate) fn call_hook(&self, data: &mut Data) {
        let hook = self.hook;
        hook(data as *mut _ as _)
    }

    pub(crate) fn send_request(&self, req: Request) -> std::result::Result<Response, KernelError> {
        // make the request
        let mut response = RunRequestResponse::Null;
        let mut data = Data::RunRequest { req, response: &mut response as _ };
        self.call_hook(&mut data);


        match response {
            RunRequestResponse::Null => Err(KernelError::text("request was not handled in kernel")),
            RunRequestResponse::AllocBuffer(len) => {
                // get the buffer
                let mut data = Data::WriteBuffer {
                    buffer: Vec::with_capacity(len)
                };
                self.call_hook(&mut data);

                let buffer = if let Data::WriteBuffer { buffer } = data {
                    buffer
                } else {
                    return Err(KernelError::text("WriteBuffer request was turned into a different type of data enum"));
                };

                if buffer.len() != len {
                    return Err(KernelError::text(&format!("Kernel wrote {} bytes while the response was {} bytes", buffer.len(), len)));
                }

                postcard::from_bytes(&buffer).unwrap()
            }
            RunRequestResponse::Response(resp) => resp
        }
    }
}
