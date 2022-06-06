#![feature(decl_macro)]

mod util;

use std::collections::HashMap;
use std::ffi::CString;
use anyhow::{anyhow, bail};
use memlib::*;
use memlib::kernel::{KernelMemoryRead, KernelMemoryWrite, MapPhysical, PhysicalMemoryRead, PhysicalMemoryWrite, TranslatePhysical};
use pelite::pe64::exports::GetProcAddress;
use pelite::pe64::PeView;
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress, LoadLibraryA};
use winutil::get_kernel_module_base;

const NT_HOOK_FUNC: &str = "NtAddAtom\0";

/// Helper macro used to convert a function pointer to reduce the amount of
/// boilerplate code needed.
pub macro call_function($addr:expr, $fn_type:ty) {{
    if $addr == 0 {
        panic!("{}", "Function pointer cannot be null!");
    }

    unsafe { core::mem::transmute::<_, $fn_type>($addr) }
}}

/// A wrapper for KernelMemoryRead and KernelMemoryWrite allowing the user to
/// perform varius kernel operations
pub struct Kernel<'a, T: KernelMemoryRead + KernelMemoryWrite + PhysicalMemoryRead + PhysicalMemoryWrite + TranslatePhysical> {
    pub memory: &'a T,
    user_hook_fn: usize,
    kernel_hook_fn: usize,
}

impl<'a, T: KernelMemoryRead + KernelMemoryWrite + PhysicalMemoryRead + PhysicalMemoryWrite + TranslatePhysical> Kernel<'a, T> {
    pub fn new(memory: &'a T) -> anyhow::Result<Self> {
        unsafe {
            let user_module = GetModuleHandleA("ntdll.dll");
            if user_module.0 == 0 {
                panic!("Failed to get module handle for ntdll.dll");
            }

            let user_hook_fn = core::mem::transmute(GetProcAddress(user_module, NT_HOOK_FUNC));
            if user_hook_fn == 0 {
                panic!("Failed to get proc address for {}", NT_HOOK_FUNC);
            }

            let kernel_base = get_kernel_module_base("ntoskrnl.exe").unwrap();
            log::debug!("Found kernel_base: {:#X}", kernel_base);
            let kernel_hook_fn = util::find_export(memory, kernel_base as u64, NT_HOOK_FUNC)?;

            Ok(Self { memory, user_hook_fn, kernel_hook_fn })
        }
    }

    /// Calls a kernel function with the specified kernel module and function.
    /// # Safety
    /// If the args are incorrect the computer may bluescreen
    pub unsafe fn call_function<F, R>(&mut self, module: &str, func: &str, callback: F) -> anyhow::Result<R>
        where F: FnOnce(usize) -> R {
        let func_address = self.get_kernel_proc(module, func)?;
        self.call_function_at(func_address, callback)
    }

    /// Calls a kernel function at the specified func_address. The callback
    /// runs with the usermode function that redirects to the specified kernel function
    /// # Safety
    /// The func_address must be a valid kernel function, or the computer will bluescreen
    pub unsafe fn call_function_at<F, R>(&mut self, func_address: usize, callback: F) -> anyhow::Result<R>
        where F: FnOnce(usize) -> R {
        log::debug!("Calling function at {:X}", func_address);

        // Write shellcode that jumps to the function we are trying to call
        let jmp_hook = [0x48, 0xb8]
            .into_iter()
            .chain(func_address.to_ne_bytes().into_iter())
            .chain([0xff, 0xe0].into_iter())
            .collect::<Vec<u8>>();

        let original_bytes = self.memory.try_read_bytes(self.kernel_hook_fn as _, jmp_hook.len()).unwrap();
        if original_bytes[0] == jmp_hook[0] && original_bytes[1] == jmp_hook[1] {
            bail!("Hook already set at {:X}", self.kernel_hook_fn);
        }

        let writer = self.memory.virtual_writer();
        writer.try_write_bytes(self.kernel_hook_fn as _, &jmp_hook).unwrap();
        let result = callback(self.user_hook_fn);
        writer.try_write_bytes(self.kernel_hook_fn as _, &original_bytes).unwrap();
        Ok(result)
    }

    unsafe fn get_kernel_proc(&mut self, module_name: &str, proc_name: &str) -> anyhow::Result<usize> {
        log::trace!("Getting kernel proc {}!{}", module_name, proc_name);
        // Get module
        let module = winutil::get_kernel_module_base(module_name)
            .ok_or_else(|| anyhow!("Failed to get module base for {}", module_name))?;

        util::find_export(self.memory, module as u64, proc_name)
    }
}
