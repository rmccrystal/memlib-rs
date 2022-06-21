use memlib::kernel::KernelMemoryRead;
use memlib::*;
use pelite::image::*;
use windows::Win32::System::SystemServices::IMAGE_NT_SIGNATURE;
use anyhow::bail;

pub unsafe fn find_export(api: &impl KernelMemoryRead, module_base: u64, function: &str) -> anyhow::Result<usize> {
    log::debug!("Finding export {} with base {:#X}", function, module_base);
    let dos_header: IMAGE_DOS_HEADER =
        api.try_read_unchecked::<IMAGE_DOS_HEADER>(module_base).unwrap();
    if dos_header.e_magic != IMAGE_DOS_SIGNATURE as u16 {
        bail!("Dos header mismatch");
    }

    let nt_header_bytes = api.read::<[u8; 0x400]>(module_base + dos_header.e_lfanew as u64);
    let nt_header: &IMAGE_NT_HEADERS64 = (nt_header_bytes.as_ptr() as *const IMAGE_NT_HEADERS64).as_ref().unwrap();
    if nt_header.Signature != IMAGE_NT_SIGNATURE {
        bail!("Invalid NT signature");
    }

    let export_entry: IMAGE_DATA_DIRECTORY =
        *nt_header.OptionalHeader.DataDirectory.as_ptr().add(IMAGE_DIRECTORY_ENTRY_EXPORT as usize);
    let export_directory = api.try_read_unchecked::<IMAGE_EXPORT_DIRECTORY>(
        module_base + export_entry.VirtualAddress as u64,
    ).unwrap();

    let address_of_names = module_base + export_directory.AddressOfNames as u64;
    let address_of_functions = module_base + export_directory.AddressOfFunctions as u64;
    let address_of_ordinals = module_base + export_directory.AddressOfNameOrdinals as u64;

    assert_ne!(export_directory.NumberOfNames, 0);

    log::trace!("Number of names: {}", export_directory.NumberOfNames);

    // Find the name
    //
    let mut index = 0;
    while index < export_directory.NumberOfNames {
        let offset =
            api.try_read_unchecked::<u32>(address_of_names + (0x4 * index) as u64).unwrap() as u64;
        let export_name = module_base + offset;

        let export_name = api.try_read_bytes(export_name, function.len()).unwrap();
        let export_name = core::str::from_utf8(&export_name);

        // Get the export or continue if invalid
        //
        let export_name = if let Ok(export_name) = export_name {
            export_name
        } else {
            continue;
        };

        // Check if the export matches
        //
        if export_name.contains(function) {
            let ordinal =
                api.try_read_unchecked::<u16>(address_of_ordinals + (0x2 * index) as u64).unwrap();
            let function = api
                .try_read_unchecked::<u32>(address_of_functions + (0x4 * ordinal) as u64).unwrap()
                as u64;

            return Ok((module_base + function) as usize);
        }

        index += 1;
    }

    bail!("Export not found");
}
