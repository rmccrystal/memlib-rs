pub mod logger;

use core::mem::MaybeUninit;
use std::time::Duration;
use dataview::Pod;

extern crate alloc;

pub trait ProcessAttach: Send {
    type ProcessType: MemoryRead + MemoryWrite + ModuleList;

    fn attach(&self, process_name: &str) -> anyhow::Result<Self::ProcessType>;
}

pub type MemoryRange = (u64, u64);

pub trait MemoryRead {
    /// Reads bytes from the process at the specified address into a buffer.
    /// Returns None if the address is not valid
    fn try_read_bytes_into(&self, address: u64, buffer: &mut [u8]) -> Option<()>;

    /// Reads bytes from the process at the specified address and returns the bytes as a Vector.
    /// Returns none if the address is not valid
    fn try_read_bytes(&self, address: u64, len: usize) -> Option<Vec<u8>> {
        let mut buf = vec![0u8; len];
        self.try_read_bytes_into(address, &mut buf).map(|_| buf)
    }

    /// Reads bytes from the process at the specified address into a value of type T.
    /// Returns None if the address is not valid
    fn try_read<T: Pod>(&self, address: u64) -> Option<T> {
        let mut buffer: MaybeUninit<T> = core::mem::MaybeUninit::zeroed();

        unsafe {
            self.try_read_bytes_into(address, buffer.assume_init_mut().as_bytes_mut())?;
            Some(buffer.assume_init())
        }
    }

    /// Reads bytes from the process at the specified address into a value of type T.
    /// Panics if the address is not valid
    fn read<T: Pod>(&self, address: u64) -> T {
        self.try_read(address).unwrap()
    }

    fn dump_memory(&self, range: MemoryRange) -> Option<Vec<u8>> {
        self.try_read_bytes(range.0, (range.1 - range.0) as usize)
    }
}

pub trait MemoryWrite {
    /// Writes bytes from the buffer into the process at the specified address.
    /// Returns None if the address is not valid
    fn try_write_bytes(&self, address: u64, buffer: &[u8]) -> Option<()>;

    /// Returns None if the address is not valid
    fn try_write<T: Pod>(&self, address: u64, buffer: &T) -> Option<()> {
        self.try_write_bytes(address, buffer.as_bytes())
    }

    /// Writes bytes to the process at the specified address with the value of type T.
    /// Panics if the address is not valid
    fn write<T: Pod>(&self, address: u64, buffer: &T) {
        self.try_write(address, buffer).unwrap()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Module {
    pub name: String,
    pub base: u64,
    pub size: u64,
}

impl Module {
    pub fn memory_range(&self) -> MemoryRange {
        (self.base, self.base + self.size)
    }
}

pub trait ModuleList {
    fn get_module_list(&self) -> Vec<Module>;

    fn get_module(&self, name: &str) -> Option<Module> {
        self.get_module_list().into_iter().find(|m| m.name.to_lowercase() == name.to_lowercase())
    }
}

pub trait ProcessInfo {
    fn process_name(&self) -> String;
    fn peb_base_address(&self) -> u64;
}

pub trait System {
    fn log(&self, text: &str);
    fn sleep(&self, duration: Duration);
    fn mouse_move(&self, dx: i32, dy: i32);
}

