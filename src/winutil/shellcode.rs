use anyhow::*;
use log::*;
use winapi::_core::ptr::null_mut;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::memoryapi::{VirtualAllocEx, WriteProcessMemory};
use winapi::um::processthreadsapi::{CreateRemoteThread, GetExitCodeThread, OpenProcess};
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::winbase::INFINITE;
use winapi::um::winnt::*;

/// Injects and runs shellcode into a process
/// Returns the return value of the shellcode or an error
pub unsafe fn inject_shellcode(code: &[u8], pid: u32) -> Result<u32> {
    let process = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);
    trace!("Opened process: {:p}", process);

    let remote_buf_addr = VirtualAllocEx(process, null_mut(), code.len(), MEM_COMMIT, PAGE_EXECUTE_READWRITE);
    if remote_buf_addr.is_null() {
        bail!("Failed to allocate remote buffer, {}", GetLastError());
    }
    trace!("Remotely allocated buffer: {:p}", remote_buf_addr);

    let mut bytes_written = 0;
    let status = WriteProcessMemory(process, remote_buf_addr, code.as_ptr() as _, code.len(), &mut bytes_written);
    if status == 0 {
        bail!("WriteProcessMemory failed: {}", GetLastError());
    }
    if bytes_written < code.len() {
        bail!("WriteProcessMemory didn't write enough bytes! bytes_written = {}, code.len() = {}", bytes_written, code.len());
    }
    trace!("Wrote {} bytes of shellcode", bytes_written);

    let mut thread_id = 0;
    let remote_thread = CreateRemoteThread(
        process,
        null_mut(),
        0,
        Some(std::mem::transmute(remote_buf_addr)),
        null_mut(),
        0,
        &mut thread_id
    );
    if remote_thread.is_null() {
        bail!("Could not create remote thread: {}", GetLastError());
    }
    trace!("Created remote thread with start address {:p}", remote_buf_addr);

    WaitForSingleObject(remote_thread, INFINITE);
    let mut exit_code = 0;
    GetExitCodeThread(remote_thread, &mut exit_code);

    trace!("Thread exited with code {}", exit_code);

    Ok(exit_code)
}