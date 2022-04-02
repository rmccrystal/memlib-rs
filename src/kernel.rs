use crate::{MemoryRead, MemoryWrite};

/// Represents a type that can load and unload a kernel exploit
pub trait LoadDriver {
    type DriverType: KernelMemoryRead + KernelMemoryWrite + MapPhysical + TranslatePhysical;

    fn load(&self) -> Option<Self::DriverType>;
    fn unload(&self) -> Option<()>;
}

/// Implementing this trait marks that a MemoryRead implementation can read from kernel memory
pub trait KernelMemoryRead: MemoryRead {}

/// Implementing this trait marks that a MemoryWrite implementation can write to kernel memory
pub trait KernelMemoryWrite: MemoryWrite {}

/// A trait for types that can call MmMapIoSpace, ZwMapViewOfSection, or any other way of mapping physical memory
/// to a virtual buffer that can be used by KernelMemoryRead and KernelMemoryWrite. Note that the virtual memory
/// created by this trait cannot be read directly
pub trait MapPhysical {
    /// Maps a physical address into virtual memory with the specified size.
    /// Returns a usize pointing to the base of the mapped memory
    /// # Safety
    /// If this function is used incorrectly the computer may blue screen
    unsafe fn map_io_space(&self, physical_address: u64, size: usize) -> Option<u64>;

    /// Unmaps a physical address mapped with map_io_space
    /// # Safety
    /// This function may blue screen if invalid information is passed in the parameters
    unsafe fn unmap_io_space(&self, virtual_address: u64, size: usize) -> Option<()>;

    /// Nicer api for mapping physical memory. Maps a physical section according to the parameters
    /// and returns a MappedPhysicalMemory struct which implements MemoryRead and MemoryWrite, allowing
    /// reading and memory of the mapped buffer (note that the addresses start at zero when reading)
    /// This struct will automatically unmap its memory when it is dropped.
    fn map_physical(&self, physical_address: u64, size: usize) -> Option<MappedPhysicalMemory<Self>>
        where Self: Sized + KernelMemoryRead + KernelMemoryWrite {
        unsafe {
            self.map_io_space(physical_address, size)
                .map(|base| MappedPhysicalMemory::new(self, base, size))
        }
    }

    /// Crates a struct which implements KernelMemoryRead and KernelMemoryWrite that uses
    /// physical memory to write to read only memory
    fn map_virtual(&self) -> VirtualTranslatePhysWrite<Self>
        where Self: Sized + KernelMemoryRead + KernelMemoryWrite + TranslatePhysical {
        VirtualTranslatePhysWrite { api: self }
    }
}

pub trait TranslatePhysical {
    fn physical_address(&self, virtual_address: usize) -> u64;
}

/// A struct allowing the writing of read only memory by translating virtual
/// address into physical addresses and writing to the physical address
pub struct VirtualTranslatePhysWrite<'a, T: MapPhysical + TranslatePhysical + KernelMemoryRead + KernelMemoryWrite> {
    api: &'a T,
}

impl<'a, T: MapPhysical + TranslatePhysical + KernelMemoryRead + KernelMemoryWrite> VirtualTranslatePhysWrite<'a, T> {
    pub fn new(api: &'a T) -> Self {
        Self { api }
    }
}

impl<'a, T: MapPhysical + TranslatePhysical + KernelMemoryRead + KernelMemoryWrite> MemoryWrite for VirtualTranslatePhysWrite<'a, T> {
    fn try_write_bytes(&self, address: u64, buffer: &[u8]) -> Option<()> {
        let phys = self.api.physical_address(address as _);
        let phys_map = self.api.map_physical(phys, buffer.len())?;
        phys_map.try_write_bytes(0, buffer)
    }
}

// You probably shouldn't use this but it's here if you really need to read virt using phys
impl<'a, T: MapPhysical + TranslatePhysical + KernelMemoryRead + KernelMemoryWrite> MemoryRead for VirtualTranslatePhysWrite<'a, T> {
    fn try_read_bytes_into(&self, address: u64, buffer: &mut [u8]) -> Option<()> {
        let phys = self.api.physical_address(address as _);
        let phys_map = self.api.map_physical(phys, buffer.len())?;
        phys_map.try_read_bytes_into(0, buffer)
    }
}

/// A struct describing a buffer of mapped physical memory generated by the MapPhysical trait
pub struct MappedPhysicalMemory<'a, T: MapPhysical + KernelMemoryRead + KernelMemoryWrite> {
    api: &'a T,
    base: u64,
    size: usize,
}

impl<'a, T: MapPhysical + KernelMemoryRead + KernelMemoryWrite> MappedPhysicalMemory<'a, T> {
    pub(crate) unsafe fn new(api: &'a T, base: u64, size: usize) -> Self {
        Self { api, base, size }
    }
}

impl<'a, T: MapPhysical + KernelMemoryRead + KernelMemoryWrite> crate::MemoryRead for MappedPhysicalMemory<'a, T> {
    fn try_read_bytes_into(&self, address: u64, buffer: &mut [u8]) -> Option<()> {
        if address as usize + buffer.len() > self.size {
            return None;
        }
        self.api.try_read_bytes_into(self.base as u64 + address, buffer)
    }
}
impl<'a, T: MapPhysical + KernelMemoryRead + KernelMemoryWrite> KernelMemoryRead for MappedPhysicalMemory<'a, T> {}

impl<'a, T: MapPhysical + KernelMemoryRead + KernelMemoryWrite> crate::MemoryWrite for MappedPhysicalMemory<'a, T> {
    fn try_write_bytes(&self, address: u64, buffer: &[u8]) -> Option<()> {
        if address as usize + buffer.len() > self.size {
            return None;
        }
        self.api.try_write_bytes(self.base as u64 + address, buffer)
    }
}

impl<'a, T: MapPhysical + KernelMemoryRead + KernelMemoryWrite> KernelMemoryWrite for MappedPhysicalMemory<'a, T> {}

impl<'a, T: MapPhysical + KernelMemoryRead + KernelMemoryWrite> Drop for MappedPhysicalMemory<'a, T> {
    fn drop(&mut self) {
        unsafe { self.api.unmap_io_space(self.base, self.size).unwrap() }
    }
}