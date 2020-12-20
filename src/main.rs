pub mod hacks;
pub mod logger;
pub mod math;
pub mod memory;
pub mod util;
pub mod system;
pub mod overlay;

#[macro_use]
pub mod macros;

fn main() {
    let handle = memory::Handle::from(
        memory::handle_interfaces::driver_handle::DriverProcessHandle::attach("hl.exe").unwrap()
    );
    let module = handle.get_module("hl.exe");
    println!("{:?}", module);
}
