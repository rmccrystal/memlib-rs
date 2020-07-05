use super::*;

use std::cmp::PartialEq;

pub struct MemoryScan<T> {
    pub matches: Vec<Pointer<T>>,
}

/// Implements cheat engine like memory scanning
/// Example
/// ```
///
/// let memory_scan: MemoryScan<u32> = MemoryScan::new();
///
/// ```
impl<T> MemoryScan<T>
where
    T: PartialEq + Sized,
{
    /// Cheat engine like memory scanning
    /// Creates a MemoryScan object containing addresses within address_range
    /// If fast_scan is set to true, it will only search values with an address dividable by 4 (recommended)
    pub fn new(address_range: (Address, Address), fast_scan: bool) -> MemoryScan<T> {
        let align_bytes = {
            if fast_scan {
                4
            } else {
                1
            }
        };

        let mut addresses = Vec::new();

        // Push valid addresses in the address_range
        for address in address_range.0..address_range.1 {
            if address % align_bytes == 0 {
                addresses.push(address);
            }
        }

        Self::from_addresses(addresses)
    }

    /// Creates a MemoryScan from a list of addresses
    pub fn from_addresses(addresses: Vec<Address>) -> MemoryScan<T> {
        MemoryScan {
            matches: addresses.iter().map(|&addr| Pointer::new(addr)).collect(),
        }
    }

    /// Filters the matches with values which match the new scan
    pub fn scan(&mut self, handle: &Box<dyn ProcessHandle>, value: T) {
        self.matches.retain(|ptr| ptr.read(&handle) == value);
    }
}
