use memlib::{MemoryRead, MemoryWrite};
use windows::Win32::Foundation::{GetLastError, HANDLE};
use windows::Win32::System::Memory::{MEM_COMMIT, MEMORY_BASIC_INFORMATION, VIRTUAL_ALLOCATION_TYPE, VirtualQueryEx};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ACCESS_RIGHTS, PROCESS_QUERY_INFORMATION};

pub struct PageVerificationAdapter<T: MemoryRead + MemoryWrite + 'static> {
    pub handle: HANDLE,
    pub api: T,
}

impl<T: MemoryRead + MemoryWrite + 'static> PageVerificationAdapter<T> {
    pub fn new(api: T, pid: u32) -> anyhow::Result<Self> {
        let handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, false, pid) }?;
        log::debug!("Opened handle to pid {} with handle {:?}, permissions PROCESS_QUERY_INFORMATION", pid, handle);
        Ok(PageVerificationAdapter { handle, api })
    }

    fn query_memory_info(&self, address: u64) -> anyhow::Result<MEMORY_BASIC_INFORMATION> {
        let mut info = MEMORY_BASIC_INFORMATION::default();
        let result = unsafe { VirtualQueryEx(self.handle, address as *mut _, &mut info, std::mem::size_of::<MEMORY_BASIC_INFORMATION>() as _) };
        if result == 0 {
            anyhow::bail!("VirtualQueryEx failed with error {:?}", unsafe { GetLastError() });
        }
        Ok(info)
    }

    pub fn is_valid_address(&self, address: u64) -> anyhow::Result<bool> {
        // TODO: Cache
        let info = self.query_memory_info(address)?;


        Ok(info.State & MEM_COMMIT != VIRTUAL_ALLOCATION_TYPE(0))
    }
}