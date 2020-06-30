// Only compile when targeting Windows
#![cfg(target_os = "windows")]

// Implements a process handle using ReadProcessMemory and WriteProcessMemory API calls

use super::*;

use winapi::shared::ntdef::{CHAR, HANDLE, LPSTR, MAKELANGID, NULL, SUBLANG_DEFAULT, TRUE};
use winapi::um::memoryapi::{ReadProcessMemory, WriteProcessMemory};
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};

use std::ffi::CString;
use std::mem;
use winapi::_core::mem::MaybeUninit;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winbase::{
    FormatMessageA, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM,
    FORMAT_MESSAGE_IGNORE_INSERTS,
};
use winapi::um::winnt::{LANG_NEUTRAL, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE};

pub struct WinAPIProcessHandle {
    // The windows handle to the process
    process_handle: HANDLE,
}

impl WinAPIProcessHandle {
    pub fn attach(process_name: &String) -> Result<Box<dyn ProcessHandle>> {
        // https://stackoverflow.com/a/865201/11639049
        // Create an empty PROCESSENTRY32 struct
        let mut entry: PROCESSENTRY32 = unsafe { mem::zeroed() };
        entry.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;

        // Take a snapshot of every process
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };

        unsafe {
            if Process32First(snapshot, &mut entry) == 1 {
                while Process32Next(snapshot, &mut entry) == 1 {
                    let current_process_name =
                        CString::from_raw(entry.szExeFile.as_mut_ptr()).into_string()?;

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
                            let mut message_buf: [i8; 512] = [0; 512];

                            // Get the error string by the code
                            let buf_len = FormatMessageA(
                                FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
                                NULL,
                                error_code,
                                MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT) as u32,
                                message_buf.as_mut_ptr(),
                                512,
                                NULL as *mut *mut i8,
                            );

                            // Create a message from the message buffer
                            let message = CString::from_raw(message_buf.as_mut_ptr())
                                .into_string()?
                                .replace("\r\n", ""); // Remove newline

                            return Err(format!(
                                "Failed to open process {}: {} (0x{:x})",
                                current_process_name, message, error_code
                            )
                            .into());
                        }

                        return Ok(Box::new(WinAPIProcessHandle { process_handle }));
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

impl ProcessHandle for WinAPIProcessHandle {
    fn read_bytes(&self, address: u64, size: usize) -> Result<Box<[u8]>> {
        unimplemented!()
    }

    fn write_bytes(&self, address: u64, bytes: &[u8]) -> Result<()> {
        unimplemented!()
    }

    fn get_module(&self, module_name: &String) -> Option<Module> {
        unimplemented!()
    }
}
