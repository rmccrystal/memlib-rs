#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum KernelRequestType {
    ReadMemory = 1,
    WriteMemory,
    GetModule,
    GetPebBase,
}

/// Contains request and response data from the kernel
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct KernelRequest {
    pub request_type: KernelRequestType,
    pub req_buf: *mut u8,
    pub status: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ReadMemory {
    pub pid: u32,
    pub address: u64,
    pub size: u64,
    pub read_buffer: *mut u8,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WriteMemory {
    pub pid: u32,
    pub address: u64,
    pub size: u64,
    pub write_buffer: *const u8,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GetModule {
    pub pid: u32,
    pub is_64_bit: i32, // 0 or 1
    pub module_name_pointer: *const u16,
    pub module_base: u64,
    pub module_size: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GetPebBase {
    pub pid: u32,
    pub peb_base: u64,
}
