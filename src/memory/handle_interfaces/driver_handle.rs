use anyhow::*;
use kernel_client::KernelHandle;
use crate::memory::{ProcessHandleInterface, ProcessInfo, Module, Address};
use crate::winutil::get_pid_by_name;

pub struct DriverProcessHandle {
    handle: KernelHandle,
    pid: u32,
    process_name: String,
}

impl DriverProcessHandle {
    pub fn attach(process_name: impl ToString) -> Result<Self> {
        let process_name = process_name.to_string();
        let handle = KernelHandle::new().unwrap();
        let pid = get_pid_by_name(&process_name).ok_or_else(|| anyhow!("Could not find process {}", &process_name))?;

        Ok(Self {
            handle,
            pid,
            process_name,
        })
    }
}

impl ProcessHandleInterface for DriverProcessHandle {
    fn read_bytes(&self, address: Address, size: usize) -> Result<Box<[u8]>> {
        let mut buf = vec![0u8; size];
        self.handle.read_memory(self.pid as u64, address as u64, &mut buf)?;
        Ok(buf.into_boxed_slice())
    }

    fn write_bytes(&self, address: Address, bytes: &[u8]) -> Result<()> {
        self.handle.write_memory(self.pid as u64, address as u64, bytes)?;
        Ok(())
    }

    fn get_module(&self, module_name: &str) -> Option<Module> {
        self.handle.module_info(self.pid as u64).unwrap()
            .iter()
            .map(|m| Module {
                base_address: m.base_address as _,
                name: m.module_name.clone(),
                size: m.size,
            })
            .find(|m| m.name.to_lowercase() == module_name.to_lowercase())
    }

    fn get_process_info(&self) -> ProcessInfo {
        let peb_base_address = self.handle.get_peb_address(self.pid as u64).unwrap();
        let bitness = self.handle.get_process_bitness(self.pid as u64).unwrap();
        ProcessInfo { process_name: self.process_name.clone(), peb_base_address , bitness, pid: self.pid}
    }
}