use memlib::logger;
use memlib::memory;
use log::*;
use std::io::{stdin, Write, stdout};
use std::fs::File;
use std::path::PathBuf;
use object::pe::ImageDosHeader;
use anyhow::*;
use std::borrow::Borrow;
use object::{Bytes, LittleEndian};
use std::mem;
use mem::size_of;

use object::read::pe::{PeFile, PeFile32};

fn read_line(prompt: &str) -> String {
    let mut output = String::new();
    println!("{}", prompt);
    let _ = stdout().flush();
    stdin().read_line(&mut output).unwrap();

    output = output.replace("\n", "");
    output = output.replace("\r", "");

    output
}

fn main() {
    logger::MinimalLogger::init(logger::LevelFilter::Trace);

    // Get process name
    // let process_name = read_line("Enter process name:");
    let process_name = "curseforge.exe";

    // Create handle
    let handle = memory::Handle::new(&process_name).unwrap();
    let module = handle.get_module(&process_name).expect("Could not find module");

    // Get output location
    // let output_location = read_line("Enter dump location:");
    let output_location = "curseforge_dump.exe";

    // Create file
    let output_location = PathBuf::from(output_location);
    let mut file = File::create(&output_location).expect("Could not create file");

    // Dump memory
    println!("Dumping {} bytes", module.size);
    let output = dump_process(&handle, &module).unwrap();
    // file.write_all(&output).unwrap();

    println!("Dumped {} to {}", process_name, output_location.as_path().to_str().unwrap());
}

pub fn dump_process(handle: &memory::Handle, module: &memory::Module) -> Result<Vec<u8>> {
    let _bytes = handle.read_bytes(module.base_address, size_of::<ImageDosHeader>()).context("Error reading dos header")?;
    let bits = handle.get_process_info().bitness;
    let dos_header = ImageDosHeader::parse(Bytes(&_bytes))?;

    let pe_header_address = module.base_address + (dos_header.e_lfanew.get(LittleEndian) as u64);
    debug!("Found PE Header address: 0x{:X}", pe_header_address);

    let dos_stub_address = module.base_address + (size_of::<ImageDosHeader>() as u64);
    let dos_stub = handle.read_bytes(
        dos_stub_address,
        ((dos_header.e_lfanew.get(LittleEndian) as u64) - size_of::<ImageDosHeader>()) as _
    ).context("Error reading dos stub")?;

    Ok(vec![])
}
