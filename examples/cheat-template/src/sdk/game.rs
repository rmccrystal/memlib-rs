use crate::PROCESS_NAME;
use memlib::memory::{Address, Handle, Module};
use std::error::Error;

pub struct FortniteContext {
    pub handle: Handle,
    pub process_module: Module,
    pub base_address: Address,
}

impl FortniteContext {
    pub fn new(handle: Handle) -> Self {
        let process_module = handle.get_module(&PROCESS_NAME.into()).unwrap();
        let base_address = process_module.base_address;
        Self {
            handle,
            process_module,
            base_address,
        }
    }
}
