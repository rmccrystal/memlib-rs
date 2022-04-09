#![feature(decl_macro)]

use alloc::string::{FromUtf16Error, FromUtf8Error};
use core::mem::MaybeUninit;
use dataview::Pod;
use std::{mem, slice};

#[cfg(feature = "kernel")]
pub mod kernel;

#[cfg(feature = "render")]
pub mod render;

pub use render::DrawExt;

#[cfg(feature = "test")]
#[macro_use]
pub mod tests;

extern crate alloc;

/// Represents a type that can attach to a process and return
/// a struct that implements MemoryRead, MemoryWrite, and ModuleList
pub trait ProcessAttach: Sized {
    /// The type of the resulting process after attaching
    type ProcessType: MemoryRead + MemoryWrite + ModuleList + ProcessInfo;

    /// Attaches to a process of name process_name. If no process is found None is returned.
    /// If there is an error internally, this function should panic
    fn attach(&self, process_name: &str) -> Option<Self::ProcessType>;

    /// Attaches to a process by a pid. If the pid does not exist, this will return None
    fn attach_pid(&self, pid: u32) -> Option<Self::ProcessType>;

    /// Attaches to a process while consuming self
    fn attach_into(self, process_name: &str) -> Option<Self::ProcessType> {
        self.attach(process_name)
    }

    /// Attaches to a process with pid while consuming self
    fn attach_into_pid(self, pid: u32) -> Option<Self::ProcessType> {
        self.attach_pid(pid)
    }
}

pub type MemoryRange = core::ops::Range<u64>;

const MAX_STRING_SIZE: usize = 0x10000;

/// Represents any type with a buffer that can be read from
#[auto_impl::auto_impl(&, & mut, Box)]
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

    /// Dumps a memory range into a Vector. If any part of the memory range is not
    /// valid, it will return None
    fn dump_memory(&self, range: MemoryRange) -> Option<Vec<u8>> {
        self.try_read_bytes(range.start, (range.end - range.start) as usize)
    }

    /// Returns true if the specified address is valid. By default reads one byte at that location
    /// and returns the success value
    fn valid_address(&self, address: u64) -> bool {
        self.try_read_bytes(address, 1).is_some()
    }

    /// Reads a string at the specified location with char length of 1.
    /// If the address is valid or there is no null terminator
    /// in MAX_STRING_SIZE characters, it will return None
    fn try_read_string(&self, address: u64) -> Option<Result<String, FromUtf8Error>> {
        const CHUNK_SIZE: usize = 0x100;

        let mut bytes = Vec::new();
        let mut buf: [u8; CHUNK_SIZE] = unsafe { core::mem::zeroed() };
        for i in (0..MAX_STRING_SIZE).into_iter().step_by(CHUNK_SIZE) {
            self.try_read_bytes_into(address + i as u64, &mut buf)?;
            if let Some(n) = buf.iter().position(|n| *n == 0u8) {
                bytes.extend_from_slice(&buf[0..n]);
                break;
            } else {
                bytes.extend_from_slice(&buf);
            }
        }

        Some(String::from_utf8(bytes))
    }

    /// Reads a wide string at the specified location with char length of 1.
    /// If the address is valid or there is no null terminator
    /// in MAX_STRING_SIZE characters, it will return None
    fn try_read_string_wide(&self, address: u64) -> Option<Result<String, FromUtf16Error>> {
        const CHUNK_SIZE: usize = 0x100;

        let mut bytes = Vec::new();
        for i in (0..MAX_STRING_SIZE).into_iter().step_by(CHUNK_SIZE * 2) {
            let buf = self.try_read::<[u16; CHUNK_SIZE]>(address + i as u64)?;
            if let Some(n) = buf.iter().position(|n| *n == 0) {
                bytes.extend_from_slice(&buf[0..n]);
                break;
            } else {
                bytes.extend_from_slice(&buf);
            }
        }

        Some(String::from_utf16(&bytes))
    }
}

/// Extension trait for supplying generic util methods for MemoryRead
pub trait MemoryReadExt: MemoryRead {
    /// Reads bytes from the process at the specified address into a value of type T.
    /// Returns None if the address is not valid
    fn try_read<T: Pod>(&self, address: u64) -> Option<T> {
        let mut buffer: MaybeUninit<T> = mem::MaybeUninit::zeroed();

        unsafe {
            self.try_read_bytes_into(address, buffer.assume_init_mut().as_bytes_mut())?;
            Some(buffer.assume_init())
        }
    }

    /// Reads any type T from the process without the restriction of Pod
    #[allow(clippy::missing_safety_doc)]
    unsafe fn try_read_unchecked<T>(&self, address: u64) -> Option<T> {
        let mut buffer: MaybeUninit<T> = mem::MaybeUninit::zeroed();

        self.try_read_bytes_into(
            address,
            slice::from_raw_parts_mut(buffer.as_mut_ptr() as _, mem::size_of::<T>()),
        )?;
        Some(buffer.assume_init())
    }

    /// Reads bytes from the process at the specified address into a value of type T.
    /// Panics if the address is not valid
    fn read<T: Pod>(&self, address: u64) -> T {
        self.try_read(address).unwrap()
    }
}

impl<T: MemoryRead> MemoryReadExt for T {}

impl MemoryReadExt for dyn MemoryRead {}

/// Represents any type with a buffer that can be written to
#[auto_impl::auto_impl(&, & mut, Box)]
pub trait MemoryWrite {
    /// Writes bytes from the buffer into the process at the specified address.
    /// Returns None if the address is not valid
    fn try_write_bytes(&self, address: u64, buffer: &[u8]) -> Option<()>;
}

/// Extension trait for supplying generic util methods for MemoryWrite
pub trait MemoryWriteExt: MemoryWrite {
    /// Returns None if the address is not valid
    fn try_write<T: Pod>(&self, address: u64, buffer: &T) -> Option<()> {
        self.try_write_bytes(address, buffer.as_bytes())
    }

    /// Writes any type T to the process without the restriction of Pod
    #[allow(clippy::missing_safety_doc)]
    unsafe fn try_write_unchecked<T>(&self, address: u64, buffer: &T) -> Option<()> {
        self.try_write_bytes(
            address,
            slice::from_raw_parts(buffer as *const T as _, mem::size_of::<T>()),
        )
    }

    /// Writes bytes to the process at the specified address with the value of type T.
    /// Panics if the address is not valid
    fn write<T: Pod>(&self, address: u64, buffer: &T) {
        self.try_write(address, buffer).unwrap()
    }
}

impl<T: MemoryWrite> MemoryWriteExt for T {}

impl MemoryWriteExt for dyn MemoryWrite {}

/// Represents a single process module with a name, base, and size
#[derive(Debug)]
#[repr(C)]
pub struct Module {
    pub name: String,
    pub base: u64,
    pub size: u64,
}

impl Module {
    /// Returns the memory range of the entire module
    pub fn memory_range(&self) -> MemoryRange {
        self.base..(self.base + self.size)
    }
}

/// Represents a type that has access to a process's modules
#[auto_impl::auto_impl(&, & mut, Box)]
pub trait ModuleList {
    /// Returns a list of all modules. If the implementor can only
    /// provide a single module based on the name, this function should panic
    fn get_module_list(&self) -> Vec<Module>;

    /// Returns a single module by name.
    /// If the module name does not exist, returns None
    fn get_module(&self, name: &str) -> Option<Module> {
        self.get_module_list()
            .into_iter()
            .find(|m| m.name.to_lowercase() == name.to_lowercase())
    }

    /// Gets the main module from the process.
    fn get_main_module(&self) -> Module;
}

/// Represents a type that can retrieve the corresponding process's name and peb base address
#[auto_impl::auto_impl(&, & mut, Box)]
pub trait ProcessInfo {
    fn process_name(&self) -> String;
    fn peb_base_address(&self) -> u64;
    fn pid(&self) -> u32;
}

/// Represents a type that allows for sending mouse inputs
#[auto_impl::auto_impl(&, & mut, Box)]
pub trait MouseMove {
    fn mouse_move(&self, dx: i32, dy: i32);
}
