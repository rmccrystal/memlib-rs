use crate::memory::handle_interfaces::winapi_handle::error_code_to_message;
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;
use winapi::shared::ntdef::NTSTATUS;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModuleInfo {
    pub base_address: u64,
    pub size: u64,
    pub module_name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum KernelError {
    Message(String),
    Status(NTSTATUS),
}

impl std::fmt::Debug for KernelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let _ = match &self {
            KernelError::Message(msg) => write!(f, "{}", msg),
            KernelError::Status(status) => match error_code_to_message(*status as _) {
                Some(error) => write!(f, "{} ({:X})", error, status),
                None => write!(f, "{:X}", status),
            },
        };
        Ok(())
    }
}

impl KernelError {
    pub fn text(text: &str) -> Self {
        Self::Message(text.to_string())
    }
}

pub type Pid = u64;

#[derive(Clone, Debug)]
pub enum Data {
    // RunRequest runs the request and returns the length
    // so the caller can create a buffer for the variable
    // length data and collect it with WriteBuffer
    RunRequest {
        req: Request,
        // number of bytes that will be returned when WriteBuffer is called
        response: *mut RunRequestResponse,
    },
    WriteBuffer {
        buffer: Vec<u8>,
    },
}

// Returned when RunRequest is returned
pub enum RunRequestResponse {
    Null,
    // the caller should allocate a buffer and call again
    AllocBuffer(usize),
    // there is no need to allocate and a response can be immediately sent
    Response(Result<Response, KernelError>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Ping,
    ModuleInfo(Pid),
    GetPebAddress(Pid),
    ReadMemory {
        pid: Pid,
        address: u64,
        size: u64,
    },
    WriteMemory {
        pid: Pid,
        address: u64,
        buf: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Pong,
    ModuleInfo(Vec<ModuleInfo>),
    PebAddress(u64),
    ReadMemory(Vec<u8>),
    WriteMemory,
}
