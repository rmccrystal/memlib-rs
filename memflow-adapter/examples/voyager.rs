use libvoyager::Voyager;
use memflow_adapter::MemflowCompat;
use memlib::*;

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let voyager = unsafe { Voyager::new() }.unwrap();
    let memflow = MemflowCompat::new(voyager)?;
    let proc = memflow.attach("Notepad.exe").unwrap();
    let mods = proc.get_module_list();
    dbg!(&mods);

    Ok(())
}