use memlib::logger;
use memlib::memory;
use std::io::{stdin, Write, stdout};
use std::fs::File;
use std::path::PathBuf;

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
    let process_name = read_line("Enter process name:");

    // Create handle
    let handle = memory::Handle::new(&process_name).unwrap();
    let module = handle.get_module(&process_name).expect("Could not find module");

    // Get output location
    let output_location = read_line("Enter dump location:");

    // Create file
    let output_location = PathBuf::from(output_location);
    let mut file = File::create(&output_location).expect("Could not create file");

    // Dump memory
    println!("Dumping {} bytes", module.size);
    let output = handle.dump_memory(module.get_memory_range());
    file.write_all(&output).unwrap();

    println!("Dumped {} to {}", process_name, output_location.as_path().to_str().unwrap());
}
