use super::{Address, read_memory};

use log::*;
use std::cmp::{PartialEq, PartialOrd};
use std::marker::PhantomData;
use std::io::BufRead;

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
        T: PartialEq + PartialOrd + Sized + Clone,
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
            matches,
            _marker: PhantomData,
        }
    }

    /// Filters the matches with values which match the new scan
    pub fn scan(&mut self, scan_value: ScanValue<T>) {
        debug!("Scanning {} addresses", self.matches.len());
        self.matches.retain(|match_item| {
            let address = match_item.0;
            let old_value = match_item.1.clone();
            let new_value = read_memory::<T>(address);
            match &scan_value {
                ScanValue::Value(value) => new_value == *value,
                ScanValue::Increased => new_value > old_value,
                ScanValue::Decreased => new_value < old_value,
                ScanValue::Static => new_value == old_value,
                ScanValue::Changed => new_value != old_value,
            }
        });

        // update the old values
        for match_item in self.matches.iter_mut() {
            match_item.1 = read_memory(match_item.0);
        }
    }
}


// Creates a new command based scan using STDIN
pub fn new_interactive_scan<T>(address_range: (Address, Address), fast_scan: bool)
    where T: PartialEq + PartialOrd + Sized + Clone + std::str::FromStr + std::fmt::Display
{
    let mut memory_scan: MemoryScan<T> = MemoryScan::new(address_range, fast_scan);

    // Scan values from stdin
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();

        let scan_value = match line.as_str() {
            "exit" => return,
            "increased" => ScanValue::Increased::<T>,
            "decreased" => ScanValue::Decreased::<T>,
            "static" => ScanValue::Static::<T>,
            "changed" => ScanValue::Changed::<T>,
            value => {
                let input = value.parse::<T>();

                // Print if the input is not a `ScanType`
                match input {
                    Err(_) => {
                        println!(
                            "[-] {} is not a valid {}",
                            line,
                            std::any::type_name::<T>()
                        );
                        continue;
                    },
                    Ok(val) => {
                        ScanValue::Value(val)
                    }
                }
            }
        };

        // Filter the scan
        memory_scan.scan(scan_value);

        println!("[+] Found {} addresses", &memory_scan.matches.len());
        if memory_scan.matches.len() < 20 {
            for scan_match in &memory_scan.matches {
                println!("\t{} at 0x{:X} (offset 0x{:X})", scan_match.1, scan_match.0, scan_match.0 - address_range.0)
            }
        }
    }
}
