use log::LevelFilter;
use math::Vector2;
use memory::handle_interfaces::driver_handle::DriverProcessHandle;
use memory::handle_interfaces::winapi_handle::WinAPIProcessHandle;
use memory::{Handle, ProcessHandleInterface};
use overlay::OverlayInterface;
use std::thread::sleep;
use util::to_hex_string;

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

    let handle = DriverProcessHandle::attach("firefox.exe").unwrap();
    let handle = Handle::from_boxed_interface(Box::new(handle));

    dbg!(handle.get_module("firefox.exe"));
}
