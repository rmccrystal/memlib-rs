use super::{Address, Handle};

use log::*;
use std::cmp::{Eq, Ord};
use std::marker::PhantomData;
use crate::memory::read_memory;

pub struct MemoryScan<T> {
    pub matches: Vec<(Address, T)>,
    _marker: PhantomData<T>,
}

pub enum ScanValue<T> {
    Value(T),
    Increased,
    Decreased,
    Static,
    Changed,
}

/// Implements cheat engine like memory scanning
impl<T> MemoryScan<T>
    where
        T: Eq + Ord + Sized + Clone,
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

        let type_size = std::mem::size_of::<T>() as Address;
        let mut addresses = Vec::new();

        // Push valid addresses in the address_range
        for address in address_range.0..(address_range.1 - type_size) {
            if address % align_bytes == 0 {
                addresses.push(address);
            }
        }

        Self::from_addresses(addresses)
    }

    /// Creates a MemoryScan from a list of addresses
    pub fn from_addresses(addresses: Vec<Address>) -> MemoryScan<T> {
        debug!("Created a memory scan with {} addresses", addresses.len());

        let matches = addresses.iter().map(|&addr| (addr, read_memory::<T>(addr))).collect();

        MemoryScan {
            matches: matches,
            _marker: PhantomData,
        }
    }

    /// Filters the matches with values which match the new scan
    pub fn scan(&mut self, handle: &Handle, scan_value: ScanValue<T>) {
        debug!("Scanning {} addresses", self.matches.len());
        self.matches.retain(|match_item| {
            let address = match_item.0;
            let old_value = match_item.1.clone();
            let new_value = handle.read_memory::<T>(address);
            match &scan_value {
                ScanValue::Value(value) => new_value == *value,
                ScanValue::Increased => new_value > old_value,
                ScanValue::Decreased => new_value < old_value,
                ScanValue::Static => new_value == old_value,
                ScanValue::Changed => new_value != old_value,
            }
        });
    }
}
