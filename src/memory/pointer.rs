use super::*;

use std::fmt;
use std::marker::PhantomData;

/// Represents a pointer to a type in external process memory
/// This has the same memory layout as an `Address`, so this can be
/// used in structs to represent pointers to a value
/// Note that GLOBAL_HANDLE must be set with set_global_handle for this to read memory
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Pointer<T> {
    pub address: Address,
    _marker: PhantomData<T>, // Store the type value (this doesn't change memory layout)
}

impl<T> Pointer<T> {
    /// Creates a new pointer at address `address` and using process handle `handle`
    pub fn new(address: Address) -> Pointer<T> {
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
