use winapi::_core::mem;
use winapi::shared::ntdef::{MAKELANGID, NULL, SUBLANG_DEFAULT};
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS};
use winapi::um::winbase::{FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS, FormatMessageA};
use winapi::um::winnt::LANG_NEUTRAL;

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
pub(crate) fn c_char_array_to_string(buff: Vec<i8>) -> String {
    let mut new_string: Vec<u8> = Vec::new();
    for c in buff {
        if c == 0i8 {
            break;
        }
        new_string.push(c as _);
    }
    String::from_utf8(new_string).unwrap()
}
