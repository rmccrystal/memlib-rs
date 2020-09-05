use super::super::*;
use super::winapi_handle::get_pid_by_name;
use log::*;
use std::ffi::{CStr, CString};
use std::time::Instant;
use tarpc::Request;
use winapi::ctypes::c_void;
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress, LoadLibraryA};

const CODE_CLIENT_REQUEST: u32 = 0x1;
const CODE_READ_MEMORY: u32 = 0x2;
const CODE_WRITE_MEMORY: u32 = 0x3;

/// Contains request and response data from the kernel
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct KernelRequest {
    code: u32,
    pid: u32,
    client_base: u32,
    address: u32,
    buffer_addr: u64,
    size: u32,
}

const HOOKED_FN_NAME: &str = "NtQueryCompositionSurfaceStatistics";

pub struct DriverProcessHandle {
    hook: extern "stdcall" fn(*mut c_void),
    pid: u32,
}

/// Basically a line by line translation of
/// https://github.com/nbqofficial/kernel-csgo/tree/master/kernel-csgo-usermode
impl DriverProcessHandle {
    pub fn attach(process_name: impl ToString) -> Result<DriverProcessHandle> {
        let process_name = process_name.to_string();

        let hook;
        let pid;

        unsafe {
            LoadLibraryA(CString::new("user32.dll")?.as_ptr());
            LoadLibraryA(CString::new("win32u.dll")?.as_ptr());
            if GetModuleHandleA(CString::new("win32u.dll")?.as_ptr()).is_null() {
                return Err("win32u.dll failed to load".into());
            }

            debug!("Loaded user32.dll and win32u.dll");

            pid =
                get_pid_by_name(&process_name).ok_or(format!("Could not find {}", process_name))?;

            debug!("Found PID for {}: {}", process_name, pid);

            hook = Self::get_hook()?;
            // hook = NtQueryCompositionSurfaceStatistics;

            debug!(
                "Found function {}: 0x{:X}",
                HOOKED_FN_NAME, hook as *const () as usize
            );
        }

        Ok(Self { pid, hook })
    }

    unsafe fn get_hook() -> Result<extern "stdcall" fn(*mut c_void)> {
        let addr = GetProcAddress(
            GetModuleHandleA(CString::new("win32u.dll")?.as_ptr()),
            CString::new(HOOKED_FN_NAME)?.as_ptr(),
        );
        Ok(std::mem::transmute(
            addr.as_mut()
                .ok_or(format!("Could not find {}", HOOKED_FN_NAME))?,
        ))
    }

    fn call_hook(&self, req: &mut KernelRequest) {
        unsafe {
            trace!("Calling hook with request {:?} ({:p})", req, req);
            let func = self.hook;
            func(req as *mut KernelRequest as _);
            trace!("Received response {:?}", req);
        }
    }
}

impl ProcessHandleInterface for DriverProcessHandle {
    fn read_bytes(&self, address: Address, size: usize) -> Result<Box<[u8]>> {
        unsafe {
            // Create a byte buffer to store the result
            let mut buf = vec![0u8; size];

            trace!(
                "Allocated buffer with length {} at 0x{:X}",
                size,
                buf.as_mut_ptr() as usize
            );

            // Create the request
            let mut req: KernelRequest = std::mem::zeroed();

            req.code = CODE_READ_MEMORY;
            req.pid = self.pid as _;
            req.address = address as _;
            req.size = size as _;
            req.buffer_addr = buf.as_mut_ptr() as _;

            self.call_hook(&mut req);

            Ok(buf.into_boxed_slice())
        }
    }

    fn write_bytes(&self, address: Address, bytes: &[u8]) -> Result<()> {
        unimplemented!()
    }

    fn get_module(&self, module_name: &String) -> Option<Module> {
        unimplemented!()
    }

    fn get_process_info(&self) -> ProcessInfo {
        unimplemented!()
    }
}
