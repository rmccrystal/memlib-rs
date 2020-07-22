// Implements a global handle so we don't have to pass a Handle struct everywhere
// Note that this doesn't work if you have multiple process handles open for some reason,
// in that case, just pass the Handle objects where they need to be

use super::Handle;

use super::{Address, BoxedErrorResult, Module, ProcessInfo};
use log::*;
use std::fmt;
use std::marker::PhantomData;

/// The global handle object
static mut GLOBAL_HANDLE: Option<Handle> = None;

/// Gets the global handle object. If it is not set, the program will panic
pub fn get_global_handle() -> &'static Handle {
    unsafe {
        match &GLOBAL_HANDLE {
            Some(handle) => handle,
            None => {
                panic!("Attempted to use the global handle object without setting it first");
            }
        }
    }
}

/// Sets the global handle
pub fn set_global_handle(handle: Handle) {
    unsafe {
        if GLOBAL_HANDLE.is_some() {
            warn!("Tried to set the global handle while it was already set");
        }
        GLOBAL_HANDLE = Some(handle);
    }
}

// ------------------------------------------------------------- //
// Here we will reimplement `Handle` methods in a global context //
// ------------------------------------------------------------- //

/// Reads memory of type T from a process. If it is successful,
/// it will return the bytes read as type T. Otherwise, it will panic.
pub fn read_memory<T>(address: Address) -> T {
    get_global_handle().read_memory(address)
}

/// Writes memory of type T to a process. If it is successful,
/// the function will return, otherwise the function will panic
pub fn write_memory<T>(address: Address, value: T) {
    get_global_handle().write_memory(address, value)
}

/// Reads an array of length `length` and type T from the process.
/// If successful, it will return the read array as a Vec<T>,
/// Otherwise, the function will panic
pub fn read_array<T>(address: Address, length: usize) -> Vec<T> {
    get_global_handle().read_array(address, length)
}

/// Dumps memory from memory_range.0 to memory_range.1
/// Returns a boxed byte slice
pub fn dump_memory(memory_range: (Address, Address)) -> Box<[u8]> {
    get_global_handle().dump_memory(memory_range)
}

/// Reads `size` bytes from a at the specified `address`.
/// If it is successful, it will return a boxed byte slice
/// Otherwise, it will return the error.
pub fn read_bytes(address: Address, size: usize) -> BoxedErrorResult<Box<[u8]>> {
    get_global_handle().read_bytes(address, size)
}

/// Write a slice of bytes to a process at the address `address`
/// Returns an error if unsuccessful
pub fn write_bytes(address: Address, bytes: &[u8]) -> BoxedErrorResult<()> {
    get_global_handle().write_bytes(address, bytes)
}

/// Gets information about a module in the form of a Module struct by name
/// If the module is found, it will return Some with the Module object,
/// Otherwise, it will return None
pub fn get_module(module_name: impl ToString) -> Option<Module> {
    get_global_handle().get_module(module_name)
}

/// Reads a null terminated i8 array string starting at `address`
/// If the string is longer than 4096 characters, it will only read
/// the first 4096 characters
pub fn read_string(address: Address) -> String {
    get_global_handle().read_string(address)
}

/// Returns a struct of process info useful in some cheats
pub fn get_process_info() -> ProcessInfo {
    get_global_handle().get_process_info()
}

/// Represents a pointer to a type in external process memory
/// This has the same memory layout as an `Address`, so this can be
/// used in structs to represent pointers to a value
/// Note that GLOBAL_HANDLE must be set with set_global_handle for this to read memory
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
    pub fn read(&self) -> T {
        get_global_handle().read_memory(self.address)
    }

    /// Writes value to address
    pub fn write(&self, value: T) {
        get_global_handle().write_memory(self.address, value)
    }
}

impl<T> fmt::Display for Pointer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {:X}", std::any::type_name::<T>(), self.address)
    }
}
