use std::ffi::{c_void};

use winapi::um::libloaderapi::{LoadLibraryA};

use types::*;

use crate::memory::*;
use crate::memory::handle_interfaces::driver_handle::driver::HOOKED_FN_NAME;
use crate::memory::handle_interfaces::winapi_handle::get_pid_by_name;

mod driver;
mod types;

pub struct DriverProcessHandle {
    hook: extern "stdcall" fn(*mut c_void),
    pid: u32,
    process_name: String,
}

/// Basically a line by line translation of
/// https://github.com/nbqofficial/kernel-csgo/tree/master/kernel-csgo-usermode
impl DriverProcessHandle {
    pub fn attach(process_name: impl ToString) -> Result<DriverProcessHandle> {
        let process_name = process_name.to_string();

        let hook;
        let pid;

        unsafe {
            Self::init_hook()?;

            pid = get_pid_by_name(&process_name).ok_or(format!("Could not find {}", process_name))?;

            debug!("Found PID for {}: {}", process_name, pid);

            hook = Self::get_hook()?;
            // hook = NtQueryCompositionSurfaceStatistics;

            debug!(
                "Found function {}: 0x{:X}",
                HOOKED_FN_NAME, hook as *const () as usize
            );
        }

        Ok(Self {
            pid,
            hook,
            process_name,
        })
    }
}

macro_rules! request {
    ($self:ident, $req:expr, $resp_type:path) => {{
        let resp = $self.send_request($req);
        match resp {
            Err(err) => Err(format!("{:?}", err).into()),
            Ok(resp) => {
                let result: $crate::memory::Result<_> = if let $resp_type(result) = resp {
                    Ok(result)
                } else {
                    Err(format!("received invalid response type").into())
                };
                result
            }
        }
    }};
}

// when we're making a request without a response buffer (ping, write, etc)
macro_rules! request_no_resp {
    ($self:ident, $req:expr, $resp_type:path) => {{
        let resp = $self.send_request($req);
        match resp {
            Err(err) => Err(format!("{:?}", err).into()),
            Ok(resp) => {
                let result: $crate::memory::Result<_> = if let $resp_type = resp {
                    Ok(())
                } else {
                    Err(format!("received invalid response type").into())
                };
                result
            }
        }
    }};
}


impl ProcessHandleInterface for DriverProcessHandle {
    fn read_bytes(&self, address: Address, size: usize) -> Result<Box<[u8]>> {
        let buf = request!(self, Request::ReadMemory {
            address: address as _,
            pid: self.pid as _,
            size: size as _
        }, Response::ReadMemory)?;

        Ok(buf.into_boxed_slice())
    }

    fn write_bytes(&self, address: Address, bytes: &[u8]) -> Result<()> {
        request_no_resp!(self, Request::WriteMemory {
            pid: self.pid as _,
            buf: bytes.to_vec(),
            address: address as _
        }, Response::WriteMemory)
    }

    fn get_module(&self, module_name: &String) -> Option<Module> {
        let module_info = request!(self, Request::ModuleInfo(self.pid as _), Response::ModuleInfo).unwrap();

        module_info.iter()
            .find(|&module| module.module_name.to_lowercase() == module_name.to_lowercase())
            .map(|module| Module { size: module.size, base_address: module.base_address as _, name: module.module_name.clone() })
    }

    fn get_process_info(&self) -> ProcessInfo {
        let peb_base_address = request!(self, Request::GetPebAddress(self.pid as _), Response::PebAddress).unwrap();

        ProcessInfo {
            peb_base_address,
            process_name: self.process_name.clone(),
        }
    }
}
