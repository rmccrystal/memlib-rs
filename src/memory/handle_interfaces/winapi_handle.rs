// Only compile when targeting Windows
#![cfg(target_os = "windows")]

// Implements a process handle using ReadProcessMemory and WriteProcessMemory API calls

use super::super::*;

use winapi::shared::ntdef::{HANDLE, MAKELANGID, NULL, SUBLANG_DEFAULT};

use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Module32First, Module32Next, Process32First, Process32Next,
    MODULEENTRY32, PROCESSENTRY32, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS,
};

use log::*;
use std::ffi::CString;
use std::mem;

use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::{ReadProcessMemory, WriteProcessMemory};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winbase::{
    FormatMessageA, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS,
};
use winapi::um::winnt::{LANG_NEUTRAL, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE};

pub struct WinAPIProcessHandle {
    process_handle: HANDLE,
    pid: u32,
    process_name: String,
}

impl WinAPIProcessHandle {
    /// Attaches to a process using OpenProcess and implements the ProcessHandle
    /// trait using ReadProcessMemory and WriteProcessMemory
    pub fn attach<'a>(process_name: impl ToString) -> Result<Box<dyn ProcessHandleInterface + 'a>> {
        let process_name = process_name.to_string();

        // https://stackoverflow.com/a/865201/11639049
        // Create an empty PROCESSENTRY32 struct
        let mut entry: PROCESSENTRY32 = unsafe { mem::zeroed() };
        entry.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;

        // Take a snapshot of every process
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };

        unsafe {
            // TODO: This doesn't include the first process
            // TODO: This doesn't have error handling for Process32First/Next. use GetLastError
            if Process32First(snapshot, &mut entry) == 1 {
                while Process32Next(snapshot, &mut entry) == 1 {
                    // Construct the process name from the bytes in the szExeFile array
                    let current_process_name = c_char_array_to_string(entry.szExeFile.to_vec());

                    // Compare the szExeFile element to the process name in lowercase
                    if current_process_name.to_lowercase() == process_name.to_lowercase() {
                        // If we get here, we have our process
                        // Create a HANDLE
                        let process_handle = OpenProcess(
                            PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION,
                            0,
                            entry.th32ProcessID,
                        );

                        if process_handle == NULL {
                            let error_code = GetLastError();
                            let message =
                                error_code_to_message(error_code).unwrap_or("".parse().unwrap());

                            return Err(format!(
                                "Failed to open process {}: {} (0x{:x})",
                                current_process_name, message, error_code
                            )
                            .into());
                        }

                        return Ok(Box::new(WinAPIProcessHandle {
                            process_handle,
                            pid: entry.th32ProcessID,
                            process_name,
                        }));
                    }
                }
            }
        };

        Err(format!("Process {} was not found", process_name).into())
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

        unsafe {
            ReadProcessMemory(
                self.process_handle,
                address as _,
                buff.as_mut_ptr() as _,
                size,
                &mut bytes_read,
            );
        }

        if bytes_read != size {
            return Err(format!(
                "ReadProcessMemory read {} bytes when it was supposed to read {}",
                &bytes_read, &size
            )
            .into());
        }

        Ok(buff)
    }

    fn write_bytes(&self, address: Address, bytes: &[u8]) -> Result<()> {
        let mut bytes_written: usize = 0;

        unsafe {
            WriteProcessMemory(
                self.process_handle,
                address as _,
                bytes.as_ptr() as _,
                bytes.len(),
                &mut bytes_written,
            );
        }

        if bytes_written != bytes.len() {
            return Err(format!(
                "WriteProcessMemory wrote {} bytes when it was supposed to write {}",
                &bytes_written,
                &bytes.len()
            )
            .into());
        }

        Ok(())
    }

    fn get_module(&self, module_name: &String) -> Option<Module> {
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

        let module = module_list.into_iter().find(|module| {
            c_char_array_to_string(module.szModule.to_vec()) == module_name.to_lowercase()
        })?;

        Some(Module {
            name: module_name.clone(),
            size: module.modBaseSize as u64,
            base_address: module.modBaseAddr as u64,
        })
    }

    fn get_process_info(&self) -> ProcessInfo {
        ProcessInfo {
            process_name: self.process_name.clone(),
            peb_base_address: 0, // Not implemented
        }
    }
}

/// Converts a Windows error code to its corresponding message.
/// If there is no message associated with the code, this will return None
fn error_code_to_message(code: u32) -> Option<String> {
    let mut message_buf: [i8; 512] = [0; 512];

    unsafe {
        // Get the error string by the code
        let buf_len = FormatMessageA(
            FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            NULL,
            code,
            MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT) as u32,
            message_buf.as_mut_ptr(),
            512,
            NULL as *mut *mut i8,
        );

        // there is no message for the error
        if buf_len == 0 {
            return None;
        }

        // Create a message from the message buffer
        let message = CString::from_raw(message_buf.as_mut_ptr())
            .into_string()
            .unwrap()
            .replace("\r\n", ""); // Remove newline

        Some(message)
    }
}

// Converts an i8 vec found in WINAPI structs to a string
fn c_char_array_to_string(buff: Vec<i8>) -> String {
    let mut new_string: Vec<u8> = Vec::new();
    for c in buff {
        if c == 0i8 {
            break;
        }
        new_string.push(c as _);
    }
    String::from_utf8(new_string).unwrap()
}
