use std::fs::File;
use std::io::Write;
use std::error::Error;

mod memory;

fn main() {
    dump_process("modernwarfare.exe").unwrap();
}

fn dump_process(process_name: impl Into<String>) -> Result<(), Box<dyn Error>>{
    let process_name = process_name.into();
    let file_name = format!("{}.dmp", &process_name);
    eprintln!("Dumping {} to {}", process_name, file_name);

    let handle = memory::KVMProcessHandle::attach(&process_name)?;
    let dump = handle.dump_module(&process_name)?;

    let mut file = File::create(&file_name)?;
    file.write_all(&dump)?;
    Ok(())
}