// Only compile when targeting Windows
#![cfg(target_os = "windows")]

// Implements a process handle using ReadProcessMemory and WriteProcessMemory API calls

use std::mem;
use anyhow::*;

use winapi::shared::ntdef::*;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::{ReadProcessMemory, WriteProcessMemory};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::tlhelp32::*;

use winapi::um::winnt::{PROCESS_ALL_ACCESS};

use super::super::*;
use crate::winutil::{get_pid_by_name, get_last_error_result, error_code_to_message, c_char_array_to_string};
use winapi::um::wow64apiset::IsWow64Process;

pub struct WinAPIProcessHandle {
    process_handle: HANDLE,
    pid: u32,
    process_name: String,
}

impl WinAPIProcessHandle {
    /// Attaches to a process using OpenProcess and implements the ProcessHandle
    /// trait using ReadProcessMemory and WriteProcessMemory
    pub fn attach(process_name: impl ToString) -> Result<Self> {
        let process_name = process_name.to_string();
        let pid = get_pid_by_name(&process_name).ok_or_else(|| anyhow!("Could not find {}", process_name))?;

        trace!("Opening pid {} with PROCESS_ALL_ACCESS", pid);
        let process_handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, 0, pid) };

        // get_last_error_result(process_handle).context("OpenProcess failed")?;
        if process_handle.is_null() {
            bail!("OpenProcess failed");
        }

        return Ok(WinAPIProcessHandle {
            process_handle,
            pid,
            process_name,
        });
    }
}

// Close the handle when the process handle is dropped
impl Drop for WinAPIProcessHandle {
    fn drop(&mut self) {
        if self.process_handle != NULL {
            unsafe { CloseHandle(self.process_handle) };
        }
    }
}

impl ProcessHandleInterface for WinAPIProcessHandle {
    fn read_bytes(&self, address: Address, size: usize) -> Result<Box<[u8]>> {
        let mut buff: Box<[u8]> = (vec![0u8; size]).into_boxed_slice();
        let mut bytes_read: usize = 0;

        let success = unsafe {
            ReadProcessMemory(
                self.process_handle,
                address as _,
                buff.as_mut_ptr() as _,
                size,
                &mut bytes_read,
            )
        };

        if success == 0 {
            get_last_error_result(buff).context("ReadProcessMemory failed")
        } else {
            Ok(buff)
        }
    }

    fn write_bytes(&self, address: Address, bytes: &[u8]) -> Result<()> {
        let mut bytes_written: usize = 0;

        let success = unsafe {
            WriteProcessMemory(
                self.process_handle,
                address as _,
                bytes.as_ptr() as _,
                bytes.len(),
                &mut bytes_written,
            )
        };

        if success == 0 {
            get_last_error_result(()).context("WriteProcessMemory failed")
        } else {
            Ok(())
        }
    }

    fn get_module(&self, module_name: &str) -> Option<Module> {
        // Create an empty MODULEENTRY32 struct
        let mut entry: MODULEENTRY32 = unsafe { mem::zeroed() };
        entry.dwSize = mem::size_of::<MODULEENTRY32>() as u32;

        // Take a snapshot of every module in the process
        let snapshot =
            unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, self.pid) };

        let mut module_list = Vec::new();

        // TODO: This doesn't include the first module
        unsafe {
            if Module32First(snapshot, &mut entry) != 1 {
                let error = GetLastError();
                let message = error_code_to_message(error);
                println!(
                    "Error calling Module32First: {} (0x{:x})",
                    message.unwrap_or("".parse().unwrap()),
                    error
                );
            }
            module_list.push(entry);
            while Module32Next(snapshot, &mut entry) == 1 {
                module_list.push(entry)
            }
        };

        let mut modules = module_list.into_iter().map(|module| {
            (c_char_array_to_string(module.szModule.to_vec()), module)
        }).collect::<Vec<_>>();

        let (module_name, module) = modules
            .into_iter()
            .find(|(name, entry)| {
                name.to_lowercase() == module_name.to_lowercase()
            })?;

        Some(Module {
            name: module_name.to_string(),
            size: module.modBaseSize as u64,
            base_address: module.modBaseAddr as Address,
        })
    }

    fn get_process_info(&self) -> ProcessInfo {
        let is_64_bit = unsafe {
            let mut ret = 0;
            IsWow64Process(self.process_handle, &mut ret);
            ret == 1
        };

        ProcessInfo {
            process_name: self.process_name.clone(),
            peb_base_address: 0, // Not implemented
            bitness: if is_64_bit { 64 } else { 32 },
            pid: self.pid
        }
    }
}
