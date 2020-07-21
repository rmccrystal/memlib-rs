use super::{Address, get_global_handle};

use std::marker::PhantomData;
use std::fmt;
use crate::memory::{read_memory, write_memory};
use log::LevelFilter::Off;

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

pub struct BasePointer<T> {
    offset: Address,
    _marker: PhantomData<T>
}

impl<T> BasePointer<T> {
    pub const fn new(offset: Address) -> BasePointer<T> {
        BasePointer{offset, _marker: PhantomData}
    }

    pub fn read(&self, base_address: Address) -> T {
        read_memory(base_address + self.offset)
    }
}

/// The same as the Pointer<T> type, except the read() and write()
/// functions add `offset` to the value read from `address`.
pub struct OffsetPointer<T, const OFFSET: Address> {
    pub address: Address,
    _marker: PhantomData<T>,
}

// https://rust-lang.github.io/rfcs/2000-const-generics.html
impl<T, const OFFSET: Address> OffsetPointer<T, OFFSET> {
    pub fn new<U, const _OFFSET: Address>(address: Address) -> OffsetPointer<T, OFFSET> {
        OffsetPointer{
            address,
            _marker: PhantomData
        }
    }

    /// Gets the address of the value we're reading / writing
    fn get_address(&self) -> Address {
        let base = read_memory::<Address>(self.address);
        base + OFFSET
    }

    /// Reads the value of the pointer
    pub fn read(&self) -> T {
        read_memory(self.get_address())
    }

    /// Writes value to address
    pub fn write(&self, value: T) {
        write_memory(self.get_address(), value)
    }
}

// pub struct IndexedPointer<T, const WIDTH: Address> {
//     pub address: Address,
//     _marker: PhantomData<T>
// }
//
// impl<T, const WIDTH: Address> IndexedPointer<T, WIDTH> {
//
// }
//
// const test: OffsetPointer<Pointer<u32>, 5>;