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
        memory::handle_interfaces::driver_handle::DriverProcessHandle::attach("notepad.exe").unwrap()
    );

    println!("{:?}", handle.read_memory::<u32>(1000000000));
}
