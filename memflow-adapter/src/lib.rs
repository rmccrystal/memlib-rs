use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use memflow::prelude::*;
use memflow_win32::prelude::*;
use memlib::*;

type MemflowKernel<T> = Win32Kernel<CachedPhysicalMemory<'static, T, DefaultCacheValidator>, CachedVirtualTranslate<DirectTranslate, DefaultCacheValidator>>;
type MemflowProcess<T> = Win32Process<VirtualDma<CachedPhysicalMemory<'static, T, DefaultCacheValidator>, CachedVirtualTranslate<DirectTranslate, DefaultCacheValidator>, Win32VirtualTranslate>>;

pub struct MemflowCompat<T: memlib::kernel::PhysicalMemoryRead + memlib::kernel::PhysicalMemoryWrite + Send + Clone + 'static> {
    pub kernel: MemflowKernel<MemflowKernelWrapper<T>>,
}

impl<T: memlib::kernel::PhysicalMemoryRead + memlib::kernel::PhysicalMemoryWrite + Send + Clone + 'static> MemflowCompat<T> {
    pub fn new(api: T) -> anyhow::Result<Self> {
        // let dtb = 0x1ae000.into();
        // let kernel_base = 0xfffff8012f200000u64.into();

        /*
        let kernel = Win32KernelBuilder::new(MemflowKernelWrapper(api))
            .build_default_caches()
            .kernel_hint(kernel_base)
            .dtb(dtb)
            .arch(ArchitectureIdent::X86(64, false))
            .build()?;

        Ok(Self { kernel })
         */
        todo!()
    }
}

/*
impl<T: memlib::kernel::PhysicalMemoryRead + memlib::kernel::PhysicalMemoryWrite + Send + Clone + 'static> memlib::GetContext for MemflowCompat<T> {
    type Context = RefCell<MemflowProcess<MemflowKernelWrapper<T>>>;

    fn get_context_from_name(&self, process_name: &str) -> Option<Self::Context> {
        self.kernel.clone().into_process(process_name).ok().map(RefCell::new)
    }

    fn get_context_from_pid(&self, pid: u32) -> Option<Self::Context> {
        self.kernel.clone().into_process_pid(pid).ok().map(RefCell::new)
    }

    fn get_current_context(&self) -> Self::Context {
        todo!()
    }
}

impl<T: memlib::kernel::PhysicalMemoryRead + memlib::kernel::PhysicalMemoryWrite + Send + Clone + 'static> memlib::MemoryReadPid for MemflowCompat<T> {
    fn try_read_bytes_into_pid(&self, ctx: &Self::Context, address: u64, buffer: &mut [u8]) -> Option<()> {
        // TODO: Handle errors
        ctx.borrow_mut().virt_mem.virt_read_raw_into(address.into(), buffer).ok()
    }
}

impl<T: memlib::kernel::PhysicalMemoryRead + memlib::kernel::PhysicalMemoryWrite + Send + Clone + 'static> memlib::MemoryWritePid for MemflowCompat<T> {
    fn try_write_bytes_pid(&self, ctx: &Self::Context, address: u64, buffer: &[u8]) -> Option<()> {
        // TODO: Handle errors
        ctx.borrow_mut().virt_mem.virt_write_raw(address.into(), buffer).ok()
    }
}

impl<T: memlib::kernel::PhysicalMemoryRead + memlib::kernel::PhysicalMemoryWrite + Send + Clone + 'static> memlib::ModuleListPid for MemflowCompat<T> {
    fn get_module_list(&self, ctx: &Self::Context) -> Vec<Module> {
        ctx.borrow_mut().module_list().unwrap().into_iter()
            .map(|m| memlib::Module { name: m.name.to_string(), base: m.base.as_u64(), size: m.size as _ })
            .collect()
    }

    fn get_main_module(&self, ctx: &Self::Context) -> Module {
        ctx.borrow_mut().main_module_info()
            .map(|m| memlib::Module { name: m.name.to_string(), base: m.base.as_u64(), size: m.size as _ })
            .unwrap()
    }
}

impl<T: memlib::kernel::PhysicalMemoryRead + memlib::kernel::PhysicalMemoryWrite + Send + Clone + 'static> memlib::ProcessInfoPid for MemflowCompat<T> {
    fn process_name(&self, ctx: &Self::Context) -> String {
        todo!()
    }

    fn peb_base_address(&self, ctx: &Self::Context) -> u64 {
        todo!()
    }

    fn pid(&self, ctx: &Self::Context) -> u32 {
        todo!()
    }
}
 */

#[derive(Clone)]
pub struct MemflowKernelWrapper<T: memlib::kernel::PhysicalMemoryRead + memlib::kernel::PhysicalMemoryWrite + Send + Clone + 'static>(pub T);

/*
impl<T: memlib::kernel::PhysicalMemoryRead + memlib::kernel::PhysicalMemoryWrite + Send + Clone + 'static> PhysicalMemory for MemflowKernelWrapper<T> {
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> memflow::Result<()> {
        for PhysicalReadData(addr, out) in data {
            let bytes_read = self.0.physical().try_read_bytes_into_chunked_fallible::<0x1000>(addr.as_u64(), out).unwrap_or(0);
            // log::info!("{} out of {} bytes read at {:#X}", bytes_read, out.len(), addr.as_u64());
        }
        Ok(())
    }

    fn phys_write_raw_list(&mut self, data: &[PhysicalWriteData]) -> memflow::Result<()> {
        for PhysicalWriteData(addr, out) in data {
            self.0.physical().try_write_bytes(addr.as_u64(), out);
        }
        Ok(())
    }

    fn metadata(&self) -> PhysicalMemoryMetadata {
        PhysicalMemoryMetadata { size: 0xFFFF_FFFF_FFFF_FFFF, readonly: false }
    }
}
 */

impl<T: memlib::kernel::PhysicalMemoryRead + memlib::kernel::PhysicalMemoryWrite + Send + Clone + 'static> PhysicalMemory for MemflowKernelWrapper<T> {
    fn phys_read_raw_iter(&mut self, data: PhysicalReadMemOps) -> Result<()> {
        unsafe {
            for CTup3(addr, _, mut out) in data.inp {
                let mut buf = [0u8; 0x1000];
                for (page_addr, mut out) in CSliceMut::from(out).page_chunks(addr.address(), 0x1000) {
                    let read_len = out.len();
                    // log::trace!("Reading {} bytes of physical memory at {:#X}", read_len, page_addr.to_umem());
                    if self.0.physical().try_read_bytes_into(page_addr.to_umem(), &mut buf[..read_len]).is_none() {
                        // log::warn!("Failed to read {} bytes of physical memory at {:#X}", read_len, page_addr.to_umem());
                    };
                    // dbg!(buf[..read_len].iter().filter(|n| **n != 0).count());
                    out.copy_from_slice(&buf[..read_len]);
                }
                // let out = out.as_slice_mut();
                // Read chunks of 0x1000 bytes at a time not reading between 0x1000 bytes boundaries
                // let mut buf = [0u8; 0x1000];
                // let addr = addr.to_umem();
                // let mut current_addr = addr.to_umem();
                // while current_addr < addr + out.len() as umem {
                //     let read_size = std::cmp::min(buf.len(), (addr + out.len()).to_umem() - current_addr);
                //     log::trace!("Reading {} bytes of physical memory at {:#X}", read_size, current_addr);
                //     self.0.physical().try_read_bytes_into(current_addr, &mut buf[..read_size]);
                //     out[current_addr.to_umem() - addr] = buf[..read_size].to_vec();
                //     current_addr += read_size as umem;
                // }
                // let bytes_read = self.0.physical().try_read_bytes_into_chunked_fallible::<0x1000>(addr.to_umem(), out).unwrap_or(0);
                // dbg!(out.iter().filter(|n| **n != 0).count());
            }
            Ok(())
        }
    }

    fn phys_write_raw_iter(&mut self, data: PhysicalWriteMemOps) -> Result<()> {
        unsafe {
            for n in data.inp {
                self.0.physical().try_write_bytes(n.0.to_umem(), n.2.as_slice());
            }
            Ok(())
        }
    }

    fn metadata(&self) -> PhysicalMemoryMetadata {
        PhysicalMemoryMetadata {
            max_address: 0x8_0000_0000i64.into(),
            real_size: 0x8_0000_0000,
            readonly: false,
            ideal_batch_size: 128,
        }
    }
}