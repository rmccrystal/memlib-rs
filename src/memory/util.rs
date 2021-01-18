use winapi::_core::mem;
use winapi::shared::ntdef::{MAKELANGID, NULL, SUBLANG_DEFAULT};
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS};
use winapi::um::winbase::{FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS, FormatMessageA};
use winapi::um::winnt::LANG_NEUTRAL;

/// Returns a PID by a process name
pub(crate) fn get_pid_by_name(name: &str) -> Option<u32> {
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
                if current_process_name.to_lowercase() == name.to_lowercase() {
                    // If we get here, we have our process
                    return Some(entry.th32ProcessID);
                }
            }
        }
    };

    None
}

/// Converts a Windows error code to its corresponding message.
/// If there is no message associated with the code, this will return None
pub(crate) fn error_code_to_message(code: u32) -> Option<String> {
    let mut message_buf: [i8; 512] = [0; 512];

    // Get the error string by the code
    let buf_len = unsafe {
        FormatMessageA(
            FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            NULL,
            code,
            MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT) as u32,
            message_buf.as_mut_ptr(),
            512,
            NULL as *mut *mut i8,
        )
    };

    // there is no message for the error
    if buf_len == 0 {
        return None;
    }

    let mut error_string = c_char_array_to_string(message_buf.to_vec());

    // Remove \n from end of string
    error_string.pop();
    // Remove \r from end of string
    error_string.pop();

    Some(error_string)
}

// Converts an i8 vec found in WINAPI structs to a string
pub fn c_char_array_to_string(buff: Vec<i8>) -> String {
    let mut new_string: Vec<u8> = Vec::new();
    for c in buff {
        if c == 0i8 {
            break;
        }
        new_string.push(c as _);
    }
    String::from_utf8(new_string).unwrap()
}
