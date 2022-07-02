use kernel_client::driver::DriverHandle;
use libvoyager::Voyager;
use memflow::prelude::*;
use memflow_adapter::{MemflowCompat, MemflowKernelWrapper};
use memlib::*;

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let voyager = unsafe { Voyager::new() }.unwrap();
    let mut conn = MemflowKernelWrapper(voyager);
    let dtb = 0x1ae000.into();
    // let kernel_base = 0xfffff8012f200000u64.into();
    dbg!(unsafe { conn.0.translate(&conn as *const _ as _, conn.0.current_dirbase().unwrap()).unwrap() });
    let tr = x86::x64::new_translator(dtb);
    // let addr = tr.virt_to_phys(&mut conn, Address::from(kernel_base)).unwrap();
    // println!("{addr:?}");

    // let proc = memflow.attach("Notepad.exe").unwrap();
    // let mods = proc.get_module_list();
    // dbg!(&mods);

    Ok(())
}