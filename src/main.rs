use log::LevelFilter;
use memlib::logger::MinimalLogger;
use memory::{Handle, MemoryScan};
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufRead, Write};

mod memory;

fn main() {
    MinimalLogger::init(LevelFilter::Debug).unwrap();

    // Create a handle to `csgo.exe`
    let handle = Handle::new("csgo.exe").unwrap();
    // Get the memory range of the `engine.dll` module
    let memory_range = handle
        .get_module(&"csgo.exe".to_string())
        .unwrap()
        .get_memory_range();

    println!("{:x}, {:x}", memory_range.0, memory_range.1);

    // Create a type alias for the type we're scanning for
    type ScanType = i32;

    // Create a new memory scan object
    let mut memory_scan: MemoryScan<ScanType> = MemoryScan::new(memory_range, false);

    // Scan values from stdin
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();

        // Try to parse input to our ScanType
        let input = line.parse::<ScanType>();

        // Print if the input is not a `ScanType`
        if input.is_err() {
            println!(
                "[-] {} is not a valid {}",
                line,
                std::any::type_name::<ScanType>()
            );
            continue;
        }

        let input = input.unwrap();

        // Filter the scan
        memory_scan.scan(&handle, input);

        println!("[+] Found {} addresses", &memory_scan.matches.len());

        // If there are less than 20 matches, print them
        if memory_scan.matches.len() < 20 {
            for ptr in &memory_scan.matches {
                println!("\t0x{:x}", ptr);
            }
        }
    }
}
