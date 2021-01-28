use winapi::_core::mem;
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS};

use anyhow::*;
use log::*;

#[derive(Clone, Debug)]
pub struct Process {
    pub pid: u32,
    pub name: String,
}

pub fn get_process_list() -> Result<Vec<Process>> {
    // https://stackoverflow.com/a/865201/11639049
    // Create an empty PROCESSENTRY32 struct
    let mut entry: PROCESSENTRY32 = unsafe { mem::zeroed() };
    entry.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;

    // Take a snapshot of every process
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };

    let mut process_list = Vec::new();

    unsafe {
        // TODO: This doesn't include the first process
        // TODO: This doesn't have error handling for Process32First/Next. use GetLastError
        if Process32First(snapshot, &mut entry) == 1 {
            while Process32Next(snapshot, &mut entry) == 1 {
                // Construct the process name from the bytes in the szExeFile array
                let name = super::c_char_array_to_string(entry.szExeFile.to_vec());
                let pid = entry.th32ProcessID;

                process_list.push(Process {
                    name,
                    pid,
                })
            }
        }
    };

    trace!("Found {} processes", process_list.len());

    Ok(process_list)
}

/// Returns a PID by a process name
pub fn get_pid_by_name(name: &str) -> Option<u32> {
    get_process_list().unwrap()
        .iter()
        .find(|&proc| proc.name.to_lowercase() == name.to_lowercase())
        .map(|proc| proc.pid)
}
