#![feature(thread_id_value)]
use kernel_client::driver::DriverHandle;
use libvoyager::Voyager;
use memlib::kernel::{PhysicalMemoryRead, PhysicalMemoryWrite, TranslatePhysical};
use memlib::*;
use memflow_adapter::MemflowCompat;

fn main() {
    pretty_env_logger::init();
    let voyager = unsafe { Voyager::new() }.unwrap();
    let driver = unsafe { DriverHandle::new() }.unwrap();
    let api = CompareApi { voyager, driver };
    let mut memflow = MemflowCompat::new(api).unwrap();
    // dbg!(std::thread::current().id());
    // dbg!(memflow.kernel.process_info_list());
}

#[derive(Clone)]
struct CompareApi {
    voyager: Voyager,
    driver: DriverHandle,
}

impl PhysicalMemoryRead for CompareApi {
    fn try_read_bytes_physical_into(&self, physical_address: u64, buffer: &mut [u8]) -> Option<()> {
        let id = std::thread::current().id().as_u64().get();
        if id != 1 {
            panic!();
        }
        let real_out = self.driver.physical().try_read_bytes(physical_address, buffer.len())?;
        let voyager_out = self.voyager.physical().try_read_bytes(physical_address, buffer.len())?;
        let differ_count = real_out.iter().zip(&voyager_out).filter(|(a, b)| a != b).count();
        if differ_count > 0 {
            let real_out_post = self.driver.physical().try_read_bytes(physical_address, buffer.len())?;
            let real_zeroed = !real_out.iter().any(|&x| x != 0);
            if real_zeroed {
                log::info!("Real output was zeroed");
            }
            else if real_out.iter().zip(&real_out_post).any(|(a, b)| a != b) {
                log::info!("Memory changed after read {:#X}, {}", physical_address, buffer.len());
            } else {
                log::info!("{} bytes difference at {:#X}, {}", differ_count, physical_address, buffer.len());
                if buffer.len() <= 8 {
                    log::info!("Hypervisor = {:#X?}, Driver = {:#X?}", &voyager_out, &real_out);
                }
            }
        } else {
            log::info!("Read {:#X}, {}", physical_address, buffer.len());
        }

        buffer.copy_from_slice(&voyager_out);

        Some(())
    }
}

impl PhysicalMemoryWrite for CompareApi {
    fn try_write_bytes_physical(&self, physical_address: u64, buffer: &[u8]) -> Option<()> {
        todo!()
    }
}