use super::super::*;
use super::winapi_handle::get_pid_by_name;
use log::*;
use std::ffi::{CStr, CString, OsStr, OsString};
use std::time::Instant;
use tarpc::Request;
use winapi::ctypes::c_void;
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress, LoadLibraryA};

mod driver;
mod types;

use crate::memory::handle_interfaces::winapi_handle::error_code_to_message;
use types::*;
use winapi::um::winbase::AddAtomA;

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
            let mut req: ReadMemory = std::mem::zeroed();

            req.size = size as _;
            req.address = address as _;
            req.pid = self.pid;
            req.read_buffer = buf.as_mut_ptr();

            let status = self.send_request(KernelRequestType::ReadMemory, &mut req);

            if status != 0 {
                return Err(format!(
                    "Sending WriteRequest failed with error {} (0x{:X})",
                    error_code_to_message(status).unwrap_or_else(|| "".into()),
                    status
                )
                .into());
            }

            Ok(buf.into_boxed_slice())
        }
    }

    fn write_bytes(&self, address: Address, bytes: &[u8]) -> Result<()> {
        let mut req: WriteMemory = unsafe { std::mem::zeroed() };

        req.pid = self.pid;
        req.address = address as _;
        req.size = bytes.len() as _;
        req.write_buffer = bytes.as_ptr();

        let status = self.send_request(KernelRequestType::WriteMemory, &mut req);

        if status != 0 {
            return Err(format!("Sending WriteRequest failed with error 0x{:X}", status).into());
        }

        Ok(())
    }

    fn get_module(&self, module_name: &String) -> Option<Module> {
        let mut req: GetModule = unsafe { std::mem::zeroed() };

        let mut module_name_ptr: Vec<u16> = module_name.encode_utf16().collect();
        module_name_ptr.push(0);

        req.pid = self.pid;
        req.module_name_pointer = module_name_ptr.as_ptr();
        req.is_64_bit = (std::mem::size_of::<Address>() == std::mem::size_of::<u64>()) as _;

        let status = self.send_request(KernelRequestType::GetModule, &mut req);

        if status != 0 {
            panic!(format!(
                "Sending GetModule request failed with error 0x{:X}",
                status
            ));
        }

        if req.module_base == 0 {
            None
        } else {
            Some(Module {
                base_address: req.module_base as _,
                size: req.module_size as _,
                name: module_name.clone(),
            })
        }
    }

    fn get_process_info(&self) -> ProcessInfo {
        unimplemented!()
    }
}
