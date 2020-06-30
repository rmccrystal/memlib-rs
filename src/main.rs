mod memory;

fn main() {
    let _handle = memory::WinAPIProcessHandle::attach(&"csrss.exe".to_owned()).unwrap();
}
