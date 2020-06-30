// Only compile when targeting linux
#![cfg(target_os = "linux")]

// Implements a process handle for reading / writing memory externally through a KVM
// https://github.com/h33p/vmread-rs

// Use everything from mod.rs
use super::*;
use vmread;

pub struct KVMProcessHandle {
    // some structs used by the vmread library
    process: vmread::WinProcess,
    c_context: vmread::sys::WinCtx,
}

impl KVMProcessHandle {
    // Attach to a running process with `process_name`
    // If successful, returns a boxed ProcessHandle trait
    pub fn attach(process_name: &String) -> Result<Box<dyn ProcessHandle>> {
        // Creating context prints some random shit so put lines around it
        println!("--------------------------------------");
        // Create the KVM context (handle to KVM)
        let context = vmread::create_context(0);
        println!("--------------------------------------");

        // Return a human readable error if it fails
        if let Err((code, message)) = context {
            return Err(format!(
                "Creating a KVM process handle failed with code {}: {} (try running as root)",
                code, message
            )
            .into());
        }

        // Get the contents if it didn't fail
        let (mut context, c_context) = context.unwrap();

        // Find the process from the process list
        let process = context
            .refresh_processes()
            .process_list
            .iter()
            .find(|p| p.name.to_lowercase() == process_name.to_lowercase())
            .ok_or(format!("Failed to find process {}", process_name))?
            .clone();

        // Return the newly created handle
        Ok(Box::new(KVMProcessHandle { c_context, process }))
    }
}

impl ProcessHandle for KVMProcessHandle {
    fn read_bytes(&self, address: Address, size: usize) -> Result<Box<[u8]>> {
        // Create a byte buffer by creating a vec of size and turning it into a slice
        // We do this so we can let the read function write to this buffer
        let mut buff: Box<[u8]> = (vec![0u8; size]).into_boxed_slice();

        // Call the vmread api directly
        let result = unsafe {
            vmread::sys::VMemRead(
                &self.c_context.process,
                self.process.proc.dirBase,
                buff.as_mut_ptr() as u64,
                address,
                size as u64,
            )
        };

        if result < 0 {
            return Err(format!("Reading memory from process failed with code {}", result).into());
        }

        Ok(buff.into())
    }

    fn write_bytes(&self, address: Address, bytes: &[u8]) -> Result<()> {
        self.process
            .write(&self.c_context, address, &Vec::from(bytes));
        Ok(())
    }

    fn get_module(&self, module_name: &String) -> Option<Module> {
        // Create clones of self so this function can be immutable
        let mut process_list = self.process.clone();
        let mut c_ctx = self.c_context.clone();

        let module = process_list
            .refresh_modules(self.c_context)
            .module_list
            .iter()
            .find(|module| module.name.to_lowercase() == module_name.to_lowercase());

        // Return None if there is no address
        if module.is_none() {
            return None;
        }
        let module = module.unwrap();

        // If everything succeeds, return the base address
        Some(Module {
            base_address: module.info.baseAddress,
            size: module.info.sizeOfModule,
            name: module_name.clone(),
        })
    }
}