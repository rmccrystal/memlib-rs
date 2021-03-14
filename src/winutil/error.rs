use anyhow::*;
use winapi::shared::ntdef::{MAKELANGID, NTSTATUS, NULL, SUBLANG_DEFAULT};
use winapi::shared::winerror::FAILED;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winbase::{FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS, FormatMessageA};
use winapi::um::winnt::LANG_NEUTRAL;

/// Calls GetLastError and returns a result with the success value
/// as Ok and the error code as Err
pub fn get_last_error_result<T>(success: T) -> Result<T> {
    match unsafe { GetLastError() } {
        0 => Ok(success),
        code => Err(anyhow!("Error code {}", code))
    }
}

/// Converts a Windows error code to its corresponding message.
/// If there is no message associated with the code, this will return None
pub fn error_code_to_message(code: u32) -> Option<String> {
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

pub(crate) trait ToError {
    fn to_err(self) -> Result<()>;
}

impl ToError for NTSTATUS {
    fn to_err(self) -> Result<()> {
        if FAILED(self) {
            // Err(std::io::Error::from_raw_os_error(self as _).into())
            Err(anyhow!(
                "{} ({:X})",
                error_code_to_message(self as _).unwrap_or_default(),
                self
            ))
        } else {
            Ok(())
        }
    }
}

impl ToError for u32 {
    fn to_err(self) -> Result<()> {
        (self as NTSTATUS).to_err()
    }
}
