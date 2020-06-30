mod memory;

fn main() {
    let handle = memory::WinAPIProcessHandle::attach("notepad.exe").unwrap();
    let module = handle.get_module(&"notepad.exe".to_string()).unwrap();
    println!("0x{:x}", module.base_address);
    dbg!(module.size);
}
