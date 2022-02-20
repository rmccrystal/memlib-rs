pub mod logger;

use core::mem::MaybeUninit;
use dataview::Pod;
use ntapi::ntldr::LDR_DATA_TABLE_ENTRY;
use ntapi::ntpebteb::PEB;
use ntapi::ntpsapi::{
    NtQueryInformationProcess, ProcessBasicInformation, PEB_LDR_DATA, PROCESS_BASIC_INFORMATION,
};
use std::ptr;
use std::time::Duration;
use widestring::U16CString;
use winapi::shared::ntdef::NT_SUCCESS;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::PROCESS_QUERY_INFORMATION;

extern crate alloc;

pub trait ProcessAttach: Send {
    type ProcessType: MemoryRead + MemoryWrite + ProcessInfo;

    fn attach(&self, process_name: &str) -> anyhow::Result<Self::ProcessType>;
}

pub type MemoryRange = (u64, u64);

pub trait MemoryRead {
    /// Reads bytes from the process at the specified address into a buffer.
    /// Returns None if the address is not valid
    fn try_read_bytes_into(&self, address: u64, buffer: &mut [u8]) -> Option<()>;

    /// Reads bytes from the process at the specified address and returns the bytes as a Vector.
    /// Returns none if the address is not valid
    fn try_read_bytes(&self, address: u64, len: usize) -> Option<Vec<u8>> {
        let mut buf = vec![0u8; len];
        self.try_read_bytes_into(address, &mut buf).map(|_| buf)
    }

    /// Reads bytes from the process at the specified address into a value of type T.
    /// Returns None if the address is not valid
    fn try_read<T: Pod>(&self, address: u64) -> Option<T> {
        let mut buffer: MaybeUninit<T> = core::mem::MaybeUninit::zeroed();

        unsafe {
            self.try_read_bytes_into(address, buffer.assume_init_mut().as_bytes_mut())?;
            Some(buffer.assume_init())
        }
    }

    /// Reads bytes from the process at the specified address into a value of type T.
    /// Panics if the address is not valid
    fn read<T: Pod>(&self, address: u64) -> T {
        self.try_read(address).unwrap()
    }

    fn dump_memory(&self, range: MemoryRange) -> Option<Vec<u8>> {
        self.try_read_bytes(range.0, (range.1 - range.0) as usize)
    }
}

pub trait MemoryWrite {
    /// Writes bytes from the buffer into the process at the specified address.
    /// Returns None if the address is not valid
    fn try_write_bytes(&self, address: u64, buffer: &[u8]) -> Option<()>;

    /// Returns None if the address is not valid
    fn try_write<T: Pod>(&self, address: u64, buffer: &T) -> Option<()> {
        self.try_write_bytes(address, buffer.as_bytes())
    }

    /// Writes bytes to the process at the specified address with the value of type T.
    /// Panics if the address is not valid
    fn write<T: Pod>(&self, address: u64, buffer: &T) {
        self.try_write(address, buffer).unwrap()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Module {
    pub name: String,
    pub base: u64,
    pub size: u64,
}

impl Module {
    pub fn memory_range(&self) -> MemoryRange {
        (self.base, self.base + self.size)
    }
}

pub trait ProcessInfo: MemoryRead {
    fn get_pid(&self) -> u32;

    fn get_module_list(&self) -> Option<Vec<Module>> {
        let peb_base = self.peb_base_address()?;

        // PEB and PEB_LDR_DATA
        //
        let peb = {
            let memory = self.try_read_bytes(peb_base, core::mem::size_of::<PEB>())?;
            unsafe { (memory.as_ptr() as *mut PEB).read_volatile() }
        };
        let peb_ldr_data = {
            let memory =
                self.try_read_bytes(peb.Ldr as u64, core::mem::size_of::<PEB_LDR_DATA>())?;
            unsafe { (memory.as_ptr() as *mut PEB_LDR_DATA).read_volatile() }
        };

        // LIST_ENTRY
        //
        let ldr_list_head = peb_ldr_data.InLoadOrderModuleList.Flink;
        let mut ldr_current_node = peb_ldr_data.InLoadOrderModuleList.Flink;

        let mut modules = Vec::new();
        loop {
            // LDR_DATA_TABLE_ENTRY
            //
            let list_entry = {
                let memory = self.try_read_bytes(
                    ldr_current_node as u64,
                    core::mem::size_of::<LDR_DATA_TABLE_ENTRY>(),
                )?;
                unsafe { (memory.as_ptr() as *mut LDR_DATA_TABLE_ENTRY).read_volatile() }
            };

            // Add the module to the list
            //
            if !list_entry.BaseDllName.Buffer.is_null()
                && !list_entry.DllBase.is_null()
                && list_entry.SizeOfImage != 0
            {
                let name = list_entry.BaseDllName;
                let size = (name.Length / 2) as _;

                let base_name = self.try_read_bytes(name.Buffer as u64, size)?;
                let base_name =
                    unsafe { U16CString::from_ptr_truncate(base_name.as_ptr() as _, size) };

                modules.push(Module {
                    name: base_name.to_string_lossy(),
                    base: list_entry.DllBase as u64,
                    size: list_entry.SizeOfImage as u64,
                });
            }

            ldr_current_node = list_entry.InLoadOrderLinks.Flink;
            if ldr_list_head as u64 == ldr_current_node as u64 {
                break;
            }
        }

        Some(modules)
    }

    fn get_module(&self, name: &str) -> Option<Module> {
        self.get_module_list()?
            .into_iter()
            .find(|m| m.name.to_lowercase() == name.to_lowercase())
    }

    fn peb_base_address(&self) -> Option<u64> {
        // Open a handle to the process
        //
        let handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, false as _, self.get_pid()) };
        if handle == INVALID_HANDLE_VALUE {
            log::error!("Failed to open handle to process");
            return None;
        }

        // Find the peb address using NtQueryInformationProcess
        //
        let mut pbi = MaybeUninit::uninit();
        if !unsafe {
            NT_SUCCESS(NtQueryInformationProcess(
                handle as _,
                ProcessBasicInformation,
                pbi.as_mut_ptr() as _,
                core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as _,
                ptr::null_mut(),
            ))
        } {
            log::error!("Failed to execute NtQueryInformationProcess");
            unsafe { CloseHandle(handle) };
            return None;
        }
        unsafe { CloseHandle(handle) };
        let pbi: PROCESS_BASIC_INFORMATION = unsafe { pbi.assume_init() };

        Some(pbi.PebBaseAddress as u64)
    }
}

pub trait System {
    fn log(&self, text: &str);
    fn sleep(&self, duration: Duration);
    fn mouse_move(&self, dx: i32, dy: i32);
}
