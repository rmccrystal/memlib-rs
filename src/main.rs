mod memory;

fn main() {
    let handle = memory::WinAPIProcessHandle::attach(&"csrss.exe".to_owned()).unwrap();
}
