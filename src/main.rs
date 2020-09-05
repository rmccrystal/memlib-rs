use log::LevelFilter;
use math::Vector2;
use memlib::util::to_hex_string;
use memory::handle_interfaces::driver_handle::DriverProcessHandle;
use memory::handle_interfaces::winapi_handle::WinAPIProcessHandle;
use memory::{Handle, ProcessHandleInterface};
use overlay::OverlayInterface;
use std::thread::sleep;

pub mod hacks;
pub mod logger;
pub mod math;
pub mod memory;
pub mod overlay;
pub mod system;
pub mod util;

#[macro_use]
pub mod macros;

fn main() {
    logger::MinimalLogger::init(LevelFilter::Trace);

    let handle = DriverProcessHandle::attach("csgo.exe").unwrap();
    // let handle = WinAPIProcessHandle::attach("notepad.exe").unwrap();
    let handle = Handle::from_boxed_interface(Box::new(handle));

    let bytes = to_hex_string(&handle.read_bytes(0x22459cce0c0, 20).unwrap());

    dbg!(bytes);
}
