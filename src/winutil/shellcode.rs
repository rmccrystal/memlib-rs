use anyhow::*;
use log::*;
use std::io;
use winapi::_core::ptr::null_mut;
use winapi::um::memoryapi::{VirtualAllocEx, WriteProcessMemory, VirtualProtect, ReadProcessMemory};
use winapi::um::processthreadsapi::{CreateRemoteThread, GetExitCodeThread, OpenProcess, GetCurrentProcess};
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::winbase::INFINITE;
use winapi::um::winnt::*;
use iced_x86::{Instruction, InstructionBlock, BlockEncoder, BlockEncoderOptions};
use winapi::shared::minwindef::DWORD;

unsafe fn remote_allocate<T>(handle: HANDLE, data: &T, allocation_type: DWORD, allocation_protect: DWORD) -> Result<usize> {
    remote_allocate_sized(handle, data as *const T as _, std::mem::size_of::<T>(), allocation_type, allocation_protect)
}

unsafe fn remote_allocate_sized(handle: HANDLE, buf_addr: *const (), size: usize, allocation_type: DWORD, allocation_protect: DWORD) -> Result<usize> {
    let buf = VirtualAllocEx(handle, null_mut(), size, allocation_type, allocation_protect);
    if buf.is_null() {
        bail!("Could not allocate remote buffer: {}", io::Error::last_os_error());
    }
    trace!("Allocated buffer at {:p}", buf);

    let mut bytes_written = 0;
    let status = WriteProcessMemory(handle, buf, buf_addr as _, size, &mut bytes_written);
    if status == 0 {
        bail!("WriteProcessMemory failed: {}", std::io::Error::last_os_error());
    }
    if bytes_written < size {
        bail!("WriteProcessMemory didn't write enough bytes! bytes_written = {}, size = {}", bytes_written, size);
    }

    Ok(buf as _)
}

/// Runs a function inside another process. The function take
/// take one argument, a buffer of type T which can be modified.
/// The function will return the return value of the function
/// and the (potentially) modified buffer
pub fn inject_func<T>(pid: u32, func: extern "C" fn(&mut T) -> u32, data: &T) -> Result<(u32, T)> {
    unsafe {
        let process = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);
        if process.is_null() {
            bail!("Could not open pid {}: {}", pid, std::io::Error::last_os_error())
        }
        trace!("Opened process: {:p}", process);

        // Allocate function
        let remote_func = remote_allocate_sized(
            process,
            func as _,
            1000,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE
        )?;
        // Allocate data
        let remote_data = remote_allocate(process, data, MEM_COMMIT, PAGE_READWRITE)?;

        let mut thread_id = 0;
        let remote_thread = CreateRemoteThread(
            process,
            null_mut(),
            0,
            Some(std::mem::transmute(remote_func)),
            remote_data as _,
            0,
            &mut thread_id,
        );
        if remote_thread.is_null() {
            bail!("Could not create remote thread: {}", std::io::Error::last_os_error());
        }
        trace!("Created remote thread with start address 0x{:X}", remote_func);

        WaitForSingleObject(remote_thread, INFINITE);
        let mut exit_code = 0;
        GetExitCodeThread(remote_thread, &mut exit_code);

        trace!("Thread exited with code 0x{:X}", exit_code);

        let mut new_data: T = std::mem::MaybeUninit::uninit().assume_init();
        ReadProcessMemory(
            process,
            remote_data as _,
            &mut new_data as *mut _ as _,
            std::mem::size_of::<T>(),
            null_mut()
        );

        Ok((exit_code, new_data))
    }
}

/// Injects iced instructions into a process
pub fn inject_instructions(instructions: &[Instruction], bitness: u32, pid: u32) -> Result<u32> {

    let block = InstructionBlock::new(&instructions, 0);
    let result = BlockEncoder::encode(bitness, block, BlockEncoderOptions::NONE)?;

    unsafe { inject_shellcode(&result.code_buffer, pid) }
}

/// Injects and runs shellcode into a process
/// Returns the return value of the shellcode or an error
pub unsafe fn inject_shellcode(code: &[u8], pid: u32) -> Result<u32> {
    let process = unsafe { OpenProcess(PROCESS_ALL_ACCESS, 0, pid) };
    if process.is_null() {
        bail!("Could not open pid {}: {}", pid, std::io::Error::last_os_error())
    }

    trace!("Opened process: {:p}", process);

    let remote_buf_addr = VirtualAllocEx(process, null_mut(), code.len(), MEM_COMMIT, PAGE_EXECUTE_READWRITE);
    if remote_buf_addr.is_null() {
        bail!("Failed to allocate remote buffer, {}", std::io::Error::last_os_error());
    }
    trace!("Remotely allocated buffer: {:p}", remote_buf_addr);

    let mut bytes_written = 0;
    let status = WriteProcessMemory(process, remote_buf_addr, code.as_ptr() as _, code.len(), &mut bytes_written);
    if status == 0 {
        bail!("WriteProcessMemory failed: {}", std::io::Error::last_os_error());
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
        &mut thread_id,
    );
    if remote_thread.is_null() {
        bail!("Could not create remote thread: {}", std::io::Error::last_os_error());
    }
    trace!("Created remote thread with start address {:p}", remote_buf_addr);

    WaitForSingleObject(remote_thread, INFINITE);
    let mut exit_code = 0;
    GetExitCodeThread(remote_thread, &mut exit_code);

    trace!("Thread exited with code 0x{:X}", exit_code);

    Ok(exit_code)
}