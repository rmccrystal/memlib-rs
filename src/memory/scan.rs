use super::*;

use std::cmp::PartialEq;

pub struct MemoryScan<T> {
    pub matches: Vec<Pointer<T>>,
}

impl<T> MemoryScan<T> where T: PartialEq + Sized {
    // Creates a new MemoryScan using the handle and a list of addresses to scan
    pub fn new(addresses: Vec<Address>) -> MemoryScan<T> {
        MemoryScan {
            matches: addresses.iter().map(|&addr| Pointer::new(addr)).collect()
        }
    }

    // Filters the matches with values which match the new scan
    pub fn scan(&mut self, handle: Box<dyn ProcessHandle>, value: T) {
        self.matches.retain(|ptr| {
            ptr.read(&handle) == value
        });
    }
}

