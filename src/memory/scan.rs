use super::{Address, Handle, Pointer};

use log::*;
use std::cmp::PartialEq;
use winapi::_core::marker::PhantomData;
use winapi::shared::winerror::PEER_E_EVENT_HANDLE_NOT_FOUND;

pub struct MemoryScan<T> {
    pub matches: Vec<Address>,
    _marker: std::marker::PhantomData<T>,
}

/// Implements cheat engine like memory scanning
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

        let type_size = std::mem::size_of::<T>() as u64;
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
        MemoryScan {
            matches: addresses,
            _marker: PhantomData,
        }
    }

    /// Filters the matches with values which match the new scan
    pub fn scan(&mut self, handle: &Handle, value: T) {
        debug!("Scanning {} addresses", self.matches.len());
        self.matches
            .retain(|&address| handle.read_memory::<T>(address) == value);
    }
}
