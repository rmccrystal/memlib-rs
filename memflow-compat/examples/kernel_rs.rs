use kernel_client::driver::DriverHandle;
use memflow::prelude::*;
use memflow_compat::MemflowCompat;
use memlib::*;
use memlib::kernel::PhysicalMemoryWrite;

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let driver = unsafe { DriverHandle::new() }?;
    let memflow = MemflowCompat::new(driver)?;
    dbg!(memflow.kernel.kernel_modules.as_ref().unwrap());
    return Ok(());
    let proc = memflow.attach("Notepad.exe").unwrap();
    let mods = proc.get_module_list();
    dbg!(&mods);

    Ok(())
}
