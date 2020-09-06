use super::*;
use crate::memory::Result;
use std::ffi::CString;
use std::fmt::Debug;
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};

impl DriverProcessHandle {
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

    pub(crate) fn call_hook(&self, req: &mut KernelRequest) {
        unsafe {
            trace!("Calling hook with request {:?} ({:p})", req, req);
            let func = self.hook;
            func(req as *mut KernelRequest as _);
            trace!("Received response {:?}", req);
        }
    }

    pub(crate) fn send_request<T: Debug>(
        &self,
        request_type: KernelRequestType,
        req: &mut T,
    ) -> u32 {
        let mut kernel_request = KernelRequest {
            request_type,
            req_buf: req as *mut T as _,
            status: 0xDEADBEEF,
        };
        trace!("Sending request to kernel: {:?}", req);
        self.call_hook(&mut kernel_request);
        trace!("Received response from kernel: {:?}", req);

        if kernel_request.status == 0xDEADBEEF {
            error!("Driver write request status was not updated");
        }

        kernel_request.status
    }
}
