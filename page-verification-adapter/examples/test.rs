use libvoyager::Voyager;
use memlib::*;
use page_verification_adapter::PageVerificationAdapter;

fn main() {
    pretty_env_logger::init();

    let voyager = unsafe { Voyager::new() }.unwrap();
    let notepad = voyager.attach_into("Notepad.exe").unwrap();
    let pid = notepad.pid();
    let adapter = PageVerificationAdapter::new(notepad.clone(), pid).unwrap();
    let main_module = notepad.get_main_module();
    dbg!(adapter.is_valid_address(main_module.base + 0x1000));
}