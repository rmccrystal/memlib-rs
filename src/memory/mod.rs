#![allow(dead_code)]

use std::marker::PhantomData;
use std::mem;
use std::ptr;
use std::slice;

mod kvm_handle;
mod winapi_handle;

pub mod scan;

// kvm_handles are only available for linux machines running a windows KVM
#[cfg(target_os = "linux")]
pub use kvm_handle::KVMProcessHandle;
#[cfg(target_os = "windows")]
pub use winapi_handle::WinAPIProcessHandle;

// Export memory scanning lib
pub use scan::*;

// Define the type we want to use for process addresses in case we want to change it later
/// A type alias for process addresses
pub type Address = u64;

// There are going to be different error types throughout
// this package, so define Result to use a boxed Error trait
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// An abstract interface for reading and writing memory to a process
/// allowing cross platform interaction with a process.
/// This is what the ProcessHandle is built off of
pub trait ProcessHandleInterface {
    /// Reads `size` bytes from a at the specified `address`.
    /// If it is successful, it will return a boxed byte slice
    /// Otherwise, it will return the error.
    fn read_bytes(&self, address: Address, size: usize) -> Result<Box<[u8]>>;

    /// Write a slice of bytes to a process at the address `address`
    /// Returns an error if unsuccessful
    fn write_bytes(&self, address: Address, bytes: &[u8]) -> Result<()>;

    /// Gets information about a module in the form of a Module struct by name
    /// If the module is found, it will return Some with the Module object,
    /// Otherwise, it will return None
    fn get_module(&self, module_name: &String) -> Option<Module>;

    /// Returns a struct of process info useful in some cheats
    fn get_process_info(&self) -> ProcessInfo;
}

pub struct ProcessInfo {
    /// The base address of the PEB. Needed in some games
    pub peb_base_address: u64,
    /// The name of the process
    pub process_name: String,
}

/// A handle to a process allowing Reading and writing memory
pub struct Handle {
    interface: Box<dyn ProcessHandleInterface>,
}

impl Handle {
    /// Creates a new Handle using the intrinsic process handle interface and the process name
    pub fn from_interface(interface: Box<dyn ProcessHandleInterface>) -> Handle {
        Handle { interface }
    }

    #[cfg(target_os = "linux")]
    /// Automatically finds the most secure method of reading / writing
    /// memory and creates a process handle using it
    ///
    /// For example, if the program was running on linux
    /// with a KVM, a KVm handle would be created
    pub fn new(process_name: impl ToString) -> Result<Handle> {
        let process_name = process_name.to_string();
        Ok(Self::from_interface(KVMProcessHandle::attach(
            &process_name,
        )?))
    }

    #[cfg(target_os = "windows")]
    /// Automatically finds the most secure method of reading / writing
    /// memory and creates a process handle using it
    ///
    /// For example, if the program was running on linux
    /// with a KVM, a KVm handle would be created
    pub fn new(process_name: impl ToString) -> Result<Handle> {
        let process_name = process_name.to_string();
        Ok(Self::from_interface(WinAPIProcessHandle::attach(
            &process_name,
        )?))
    }

    /// Reads memory of type T from a process. If it is successful,
    /// it will return the bytes read as type T. Otherwise, it will panic.
    pub fn read_memory<T>(&self, address: Address) -> T {
        // Get size of the type
        let size = mem::size_of::<T>();
        let bytes = self
            .interface
            .read_bytes(address, size)
            .expect("Error reading bytes from process");
        // Convert the raw bytes into the type we need to return
        let value = unsafe {
            // We do this by casting the pointer to the bytes as a pointer to T
            ptr::read(bytes.as_ptr() as *const _)
        };

        value
    }

    /// Writes memory of type T to a process. If it is successful,
    /// the function will return, otherwise the function will panic
    pub fn write_memory<T>(&self, address: Address, value: T) {
        let size = mem::size_of::<T>();
        // Create a byte buffer from the type
        // https://stackoverflow.com/a/42186553
        let buff = unsafe { slice::from_raw_parts((&value as *const T) as *const u8, size) };

        self.write_bytes(address, buff)
            .expect("Failed to write memory to process")
    }

    /// Reads an array of length `length` and type T from the process.
    /// If successful, it will return the read array as a Vec<T>,
    /// Otherwise, the function will panic
    pub fn read_array<T>(&self, address: Address, length: usize) -> Vec<T> {
        let size = std::mem::size_of::<T>() as u32;
        // Creates an array lf values for our result
        let mut values = Vec::new();

        // Read memory at each address
        for i in 0..length {
            // Multiply index by size to get the pointer for the index
            let address = address + (i * size as usize) as u64;
            values.push(self.read_memory(address));
        }

        // Return the values
        values
    }

    /// Dumps the contents of a module by `module_name` to a byte vec.
    /// If the module is not found or there is an error reading memory, it will return an Error.
    /// otherwise, it will return the dump
    pub fn dump_module(&self, module_name: impl Into<String>) -> Result<Box<[u8]>> {
        let module_name = module_name.into();
        let module = self
            .interface
            .get_module(&module_name)
            .ok_or_else(|| format!("Could not find module {}", module_name))?;
        let mut buffer: Vec<u8> = Vec::new();

        // The address the module ends
        let module_end_address = module.base_address + module.size;

        // The amount of bytes to be read at a time
        let chunk_size: usize = 4096;

        // The current memory location we are reading
        let mut current_offset: Address = module.base_address;

        loop {
            // The current offset should never be greater than the module_end_address
            if current_offset > module_end_address {
                dbg!(current_offset, module_end_address);
                panic!("dump_module attempted to read invalid memory")
            }
            if current_offset == module_end_address {
                break;
            }
            // Create the size based on the current offset
            let read_size = {
                // If we would read memory which is out of bounds, resize the read_size accordingly
                if current_offset + chunk_size as u64 > module_end_address {
                    (module_end_address - current_offset) as usize
                } else {
                    chunk_size
                }
            };

            let memory = self.read_bytes(current_offset, read_size)?;

            // Append the slice of memory to the buffer
            buffer.extend_from_slice(&memory);

            current_offset += read_size as u64;
        }

        Ok(buffer.into_boxed_slice())
    }

    // -------------------------------------------------------- //
    // Implement the intrinsic `ProcessHandleInterface` methods //
    // -------------------------------------------------------- //

    /// Reads `size` bytes from a at the specified `address`.
    /// If it is successful, it will return a boxed byte slice
    /// Otherwise, it will return the error.
    pub fn read_bytes(&self, address: Address, size: usize) -> Result<Box<[u8]>> {
        self.interface.read_bytes(address, size)
    }

    /// Write a slice of bytes to a process at the address `address`
    /// Returns an error if unsuccessful
    pub fn write_bytes(&self, address: Address, bytes: &[u8]) -> Result<()> {
        self.interface.write_bytes(address, bytes)
    }

    /// Gets information about a module in the form of a Module struct by name
    /// If the module is found, it will return Some with the Module object,
    /// Otherwise, it will return None
    pub fn get_module(&self, module_name: &String) -> Option<Module> {
        self.interface.get_module(module_name)
    }

    /// Returns a struct of process info useful in some cheats
    pub fn get_process_info(&self) -> ProcessInfo {
        self.interface.get_process_info()
    }
}

/// Defines information about a module
pub struct Module {
    /// The image base address
    pub base_address: Address,
    /// Size in bytes of the module
    pub size: u64,
    /// The name of the module
    pub name: String,
}

impl Module {
    /// Returns the range of memory for the entire module
    pub fn get_memory_range(&self) -> (Address, Address) {
        (self.base_address, self.base_address + self.size)
    }
}

/// Represents a pointer to a type in external process memory
/// This has the same memory layout as an `Address`, so this can be
/// used in structs to represent pointers to a value
#[repr(C)]
pub struct Pointer<T> {
    pub address: Address,
    _marker: PhantomData<T>, // Store the type value (this doesn't change memory layout)
}

impl<T> Pointer<T> {
    /// Creates a new pointer at address `address` and using process handle `handle`
    pub fn new<U>(address: Address) -> Pointer<T> {
        Pointer {
            address,
            _marker: PhantomData,
        }
    }

    /// Reads the value of the pointer
    pub fn read(&self, handle: &crate::memory::Handle) -> T {
        handle.read_memory(self.address)
    }

    /// Writes value to address
    pub fn write(&self, value: T, handle: &Handle) {
        handle.write_memory(self.address, value)
    }
}
