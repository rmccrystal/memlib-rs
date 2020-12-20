use std::ffi::{c_void, CString};
use winapi::um::libloaderapi::{LoadLibraryA, GetModuleHandleA};
use crate::memory::*;
use crate::memory::handle_interfaces::winapi_handle::get_pid_by_name;
use crate::memory::handle_interfaces::driver_handle::driver::HOOKED_FN_NAME;
use types::*;

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

impl ProcessHandleInterface for DriverProcessHandle {
    fn read_bytes(&self, address: Address, size: usize) -> Result<Box<[u8]>> {
        unimplemented!()
    }

    fn write_bytes(&self, address: Address, bytes: &[u8]) -> Result<()> {
        unimplemented!()
    }

    fn get_module(&self, module_name: &String) -> Option<Module> {
        let resp = self.send_request(Request::ModuleInfo(self.pid as _)).unwrap();
        let module_info;
        if let Response::ModuleInfo(info) = resp {
            module_info = info;
        } else {
            panic!("received invalid response type");
        }

        for module in &module_info {
            println!("{}", module.module_name);
        }

        module_info.iter()
            .find(|&module| module.module_name.to_lowercase() == module_name.to_lowercase())
            .map(|module| Module{size: module.size, base_address: module.base_address as _, name: module.module_name.clone()})
    }

    fn get_process_info(&self) -> ProcessInfo {
        let resp = self.send_request(Request::GetPebAddress(self.pid as _)).unwrap();
        let peb_base_address;
        if let Response::PebAddress(base) = resp {
            peb_base_address = base;
        } else {
            panic!("received invalid response type");
        }

        ProcessInfo {
            peb_base_address,
            process_name: self.process_name.clone(),
        }
    }
}
