use core::mem::MaybeUninit;
use dataview::Pod;
use ntapi::ntldr::LDR_DATA_TABLE_ENTRY;
use ntapi::ntpebteb::PEB;
use ntapi::ntpsapi::{
    NtQueryInformationProcess, ProcessBasicInformation, PEB_LDR_DATA, PROCESS_BASIC_INFORMATION,
};
use std::ptr;
use std::{mem, slice};
use widestring::U16CString;
use winapi::shared::ntdef::NT_SUCCESS;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::PROCESS_QUERY_INFORMATION;

extern crate alloc;

/// Represents a type that can attach to a process and return
/// a struct that implements MemoryRead, MemoryWrite, and ModuleList
pub trait ProcessAttach {
    /// The type of the resulting process after attaching
    type ProcessType: MemoryRead + MemoryWrite + ProcessInfo;

    /// Attaches to a process of name process_name. If no process is found None is returned.
    /// If there is an error internally, this function should panic
    fn attach(&self, process_name: &str) -> Option<Self::ProcessType>;
}

pub type MemoryRange = (u64, u64);

/// Represents any type with a buffer that can be read from
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

    /// Dumps a memory range into a Vector. If any part of the memory range is not
    /// valid, it will return None
    fn dump_memory(&self, range: MemoryRange) -> Option<Vec<u8>> {
        self.try_read_bytes(range.0, (range.1 - range.0) as usize)
    }
}

/// Extension trait for supplying generic util methods for MemoryRead
pub trait MemoryReadExt: MemoryRead {
    /// Reads bytes from the process at the specified address into a value of type T.
    /// Returns None if the address is not valid
    fn try_read<T: Pod>(&self, address: u64) -> Option<T> {
        let mut buffer: MaybeUninit<T> = mem::MaybeUninit::zeroed();

        unsafe {
            self.try_read_bytes_into(address, buffer.assume_init_mut().as_bytes_mut())?;
            Some(buffer.assume_init())
        }
    }

    /// Reads any type T from the process without the restriction of Pod
    #[allow(clippy::missing_safety_doc)]
    unsafe fn try_read_unchecked<T>(&self, address: u64) -> Option<T> {
        let mut buffer: MaybeUninit<T> = mem::MaybeUninit::zeroed();

        self.try_read_bytes_into(
            address,
            slice::from_raw_parts_mut(buffer.as_mut_ptr() as _, mem::size_of::<T>()),
        )?;
        Some(buffer.assume_init())
    }

    /// Reads bytes from the process at the specified address into a value of type T.
    /// Panics if the address is not valid
    fn read<T: Pod>(&self, address: u64) -> T {
        self.try_read(address).unwrap()
    }
}

impl<T: MemoryRead> MemoryReadExt for T {}
impl MemoryReadExt for dyn MemoryRead {}

/// Represents any type with a buffer that can be written to
pub trait MemoryWrite {
    /// Writes bytes from the buffer into the process at the specified address.
    /// Returns None if the address is not valid
    fn try_write_bytes(&self, address: u64, buffer: &[u8]) -> Option<()>;
}

/// Extension trait for supplying generic util methods for MemoryWrite
pub trait MemoryWriteExt: MemoryWrite {
    /// Returns None if the address is not valid
    fn try_write<T: Pod>(&self, address: u64, buffer: &T) -> Option<()> {
        self.try_write_bytes(address, buffer.as_bytes())
    }

    /// Writes any type T to the process without the restriction of Pod
    #[allow(clippy::missing_safety_doc)]
    unsafe fn try_write_unchecked<T>(&self, address: u64, buffer: &T) -> Option<()> {
        self.try_write_bytes(
            address,
            slice::from_raw_parts(buffer as *const T as _, mem::size_of::<T>()),
        )
    }

    /// Writes bytes to the process at the specified address with the value of type T.
    /// Panics if the address is not valid
    fn write<T: Pod>(&self, address: u64, buffer: &T) {
        self.try_write(address, buffer).unwrap()
    }
}

impl<T: MemoryWrite> MemoryWriteExt for T {}
impl MemoryWriteExt for dyn MemoryWrite {}

/// Represents a single process module with a name, base, and size
#[derive(Debug)]
#[repr(C)]
pub struct Module {
    pub name: String,
    pub base: u64,
    pub size: u64,
}

impl Module {
    /// Returns the memory range of the entire module
    pub fn memory_range(&self) -> MemoryRange {
        (self.base, self.base + self.size)
    }
}

pub trait ProcessInfo: MemoryRead {
    fn process_id(&self) -> u32;

    fn process_name(&self) -> String;

    fn peb_base_address(&self) -> Option<u64> {
        // Open a handle to the process
        //
        let handle =
            unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, false as _, self.process_id()) };
        if handle == INVALID_HANDLE_VALUE {
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
            unsafe { CloseHandle(handle) };
            return None;
        }
        unsafe { CloseHandle(handle) };
        let pbi: PROCESS_BASIC_INFORMATION = unsafe { pbi.assume_init() };

        Some(pbi.PebBaseAddress as u64)
    }

    /// Returns a list of all modules.
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
                let size = name.MaximumLength as usize;

                let base_name = self.try_read_bytes(name.Buffer as u64, size)?;
                let base_name = unsafe { U16CString::from_ptr_str(base_name.as_ptr() as _) };

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

    /// Returns a single module by name.
    fn get_module(&self, name: &str) -> Option<Module> {
        self.get_module_list()?
            .into_iter()
            .find(|m| m.name.to_lowercase() == name.to_lowercase())
    }

    /// Gets the main module from the process.
    /// Panics if the main module couldn't be found
    fn get_main_module(&self) -> Module {
        self.get_module(self.process_name().as_str()).unwrap()
    }
}

/// Represents a type that allows for sending mouse inputs
pub trait MouseMove {
    fn mouse_move(&self, dx: i32, dy: i32);
}
