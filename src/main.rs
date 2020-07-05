use memlib_rs::memory::scan::MemoryScan;
use memlib_rs::memory::ProcessHandle;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufRead, Write};
use winapi::_core::iter::Scan;

mod memory;

fn main() {
    // Create a handle to `csgo.exe`
    let handle = memory::WinAPIProcessHandle::attach("csgo.exe").unwrap();
    // Get the memory range of the `engine.dll` module
    let memory_range = handle
        .get_module(&"engine.dll".to_string())
        .unwrap()
        .get_memory_range();

    // Create a type alias for the type we're scanning for
    type ScanType = i32;

    // Create a new memory scan object
    let mut memory_scan: MemoryScan<ScanType> = MemoryScan::new(memory_range, true);

    // Scan values from stdin
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();

        // Try to parse input to our ScanType
        let input = line.parse::<ScanType>();

        // Print if the input is not a `ScanType`
        if let Err(_) = input {
            println!(
                "[-] {} is not a valid {}",
                line,
                std::any::type_name::<ScanType>()
            )
        }

        let input = input.unwrap();

        // Filter the scan
        memory_scan.scan(&handle, input);

        println!("[+] Found {} addresses", memory_scan.matches.len());

        // If there are less than 20 matches, print them
        if memory_scan.matches.len() < 20 {
            for ptr in memory_scan.matches {
                println!("\t0x{:x}", ptr.address);
            }
        }
    }
}

fn dump_process(handle: &Box<dyn ProcessHandle>) -> Result<(), Box<dyn Error>> {
    let process_name = handle.get_process_info().process_name;
    let file_name = format!("{}.dmp", &process_name);
    eprintln!("Dumping {} to {}", process_name, file_name);

    let dump = handle.dump_module(&process_name)?;

    let mut file = File::create(&file_name)?;
    file.write_all(&dump)?;
    Ok(())
}
