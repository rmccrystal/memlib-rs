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

/// A trait that represents a type that can read physical memory
pub trait PhysicalMemoryRead {
    /// Reads bytes at a physical address into the buffer
    fn try_read_bytes_physical_into(&self, physical_address: u64, buffer: &mut [u8]) -> Option<()>;

    /// A utility type that transforms Self into a type that implements memlib::MemoryRead
    /// so memlib's utility functions can be used
    fn physical_reader(&self) -> PhysicalMemoryReader<Self>
        where Self: Sized {
        PhysicalMemoryReader::new(self)
    }

    /// Returns a type that uses the TranslatePhysical implementation to read virtual addresses using the physical memory reader
    fn virtual_reader(&self) -> VirtualMemoryReader<Self>
        where Self: Sized + TranslatePhysical {
        VirtualMemoryReader::new(self)
    }
}

pub struct VirtualMemoryReader<'a, T: PhysicalMemoryRead + TranslatePhysical>(&'a T);

impl<'a, T: PhysicalMemoryRead + TranslatePhysical> VirtualMemoryReader<'a, T> {
    fn new(api: &'a T) -> Self {
        Self(api)
    }
}

impl<'a, T: PhysicalMemoryRead + TranslatePhysical> MemoryRead for VirtualMemoryReader<'a, T> {
    fn try_read_bytes_into(&self, address: u64, buffer: &mut [u8]) -> Option<()> {
        let physical_address = self.0.physical_address(address)?;
        self.0.try_read_bytes_physical_into(physical_address, buffer)
    }
}

impl<'a, T: PhysicalMemoryRead + TranslatePhysical> KernelMemoryRead for VirtualMemoryReader<'a, T> {}

pub struct PhysicalMemoryReader<'a, T: PhysicalMemoryRead>(&'a T);

impl<'a, T: PhysicalMemoryRead> PhysicalMemoryReader<'a, T> {
    pub fn new(api: &'a T) -> Self {
        Self(api)
    }
}

impl<'a, T: PhysicalMemoryRead> MemoryRead for PhysicalMemoryReader<'a, T> {
    fn try_read_bytes_into(&self, address: u64, buffer: &mut [u8]) -> Option<()> {
        self.0.try_read_bytes_physical_into(address, buffer)
    }
}

/// A trait that represents a type that can write physical memory
pub trait PhysicalMemoryWrite {
    /// Writes bytes at a physical address from the buffer
    fn try_write_bytes_physical(&self, physical_address: u64, buffer: &[u8]) -> Option<()>;

    /// A utility type that transforms Self into a type that implements memlib::MemoryWrite
    /// so memlib's utility functions can be used
    fn physical_writer(&self) -> PhysicalMemoryWriter<Self>
        where Self: Sized {
        PhysicalMemoryWriter::new(self)
    }

    /// A utility type that transforms Self into a type that implements memlib::MemoryWrite + memlib::MemoryRead
    /// so memlib's utility functions can be used
    fn physical(&self) -> PhysicalMemory<Self>
        where Self: Sized + PhysicalMemoryRead {
        PhysicalMemory::new(self)
    }

    /// A utility type that transforms Self into a type that writes virtual memory using physical memory
    fn virtual_writer(&self) -> VirtualMemoryWriter<Self>
        where Self: Sized + TranslatePhysical {
        VirtualMemoryWriter::new(self)
    }

    /// A utility type that transforms Self into a type that reads and writes virtual memory using physical memory and TranslatePhysical
    fn virtual_memory(&self) -> VirtualMemory<Self>
        where Self: Sized + TranslatePhysical + PhysicalMemoryRead {
        VirtualMemory::new(self)
    }
}

pub struct VirtualMemory<'a, T: PhysicalMemoryRead + PhysicalMemoryWrite + TranslatePhysical>(&'a T);

impl<'a, T: PhysicalMemoryRead + PhysicalMemoryWrite + TranslatePhysical> VirtualMemory<'a, T> {
    pub fn new(api: &'a T) -> Self {
        Self(api)
    }
}

impl<'a, T: PhysicalMemoryRead + PhysicalMemoryWrite + TranslatePhysical> MemoryRead for VirtualMemory<'a, T> {
    fn try_read_bytes_into(&self, address: u64, buffer: &mut [u8]) -> Option<()> {
        let physical_address = self.0.physical_address(address)?;
        self.0.try_read_bytes_physical_into(physical_address, buffer)
    }
}

impl<'a, T: PhysicalMemoryRead + PhysicalMemoryWrite + TranslatePhysical> MemoryWrite for VirtualMemory<'a, T> {
    fn try_write_bytes(&self, address: u64, buffer: &[u8]) -> Option<()> {
        let physical_address = self.0.physical_address(address)?;
        self.0.try_write_bytes_physical(physical_address, buffer)
    }
}

pub struct VirtualMemoryWriter<'a, T: PhysicalMemoryWrite + TranslatePhysical>(&'a T);

impl<'a, T: PhysicalMemoryWrite + TranslatePhysical> VirtualMemoryWriter<'a, T> {
    pub fn new(api: &'a T) -> Self {
        Self(api)
    }
}

impl<'a, T: PhysicalMemoryWrite + TranslatePhysical> MemoryWrite for VirtualMemoryWriter<'a, T> {
    fn try_write_bytes(&self, address: u64, buffer: &[u8]) -> Option<()> {
        let physical_address = self.0.physical_address(address)?;
        self.0.try_write_bytes_physical(physical_address, buffer)
    }
}

impl<'a, T: PhysicalMemoryWrite + TranslatePhysical> KernelMemoryWrite for VirtualMemoryWriter<'a, T> {}

pub struct PhysicalMemoryWriter<'a, T: PhysicalMemoryWrite>(&'a T);

impl<'a, T: PhysicalMemoryWrite> PhysicalMemoryWriter<'a, T> {
    pub fn new(api: &'a T) -> Self {
        Self(api)
    }
}

impl<'a, T: PhysicalMemoryWrite> MemoryWrite for PhysicalMemoryWriter<'a, T> {
    fn try_write_bytes(&self, address: u64, buffer: &[u8]) -> Option<()> {
        self.0.try_write_bytes_physical(address, buffer)
    }
}

pub struct PhysicalMemory<'a, T: PhysicalMemoryRead + PhysicalMemoryWrite>(&'a T);

impl<'a, T: PhysicalMemoryRead + PhysicalMemoryWrite> PhysicalMemory<'a, T> {
    pub fn new(api: &'a T) -> Self {
        Self(api)
    }
}

impl<'a, T: PhysicalMemoryRead + PhysicalMemoryWrite> MemoryRead for PhysicalMemory<'a, T> {
    fn try_read_bytes_into(&self, address: u64, buffer: &mut [u8]) -> Option<()> {
        self.0.try_read_bytes_physical_into(address, buffer)
    }
}

impl<'a, T: PhysicalMemoryRead + PhysicalMemoryWrite> MemoryWrite for PhysicalMemory<'a, T> {
    fn try_write_bytes(&self, address: u64, buffer: &[u8]) -> Option<()> {
        self.0.try_write_bytes_physical(address, buffer)
    }
}

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
        where
            Self: Sized + KernelMemoryRead + KernelMemoryWrite,
    {
        unsafe {
            self.map_io_space(physical_address, size)
                .map(|base| MappedPhysicalMemory::new(self, base, size))
        }
    }

    /// Converts the specified virtual address and then calls `map_physical`.
    fn map_virtual(&self, virtual_address: u64, size: usize) -> Option<MappedPhysicalMemory<Self>>
        where
            Self: Sized + KernelMemoryRead + KernelMemoryWrite + TranslatePhysical,
    {
        let phys = self.physical_address(virtual_address)?;
        self.map_physical(phys, size)
    }
}

impl<T: MapPhysical + KernelMemoryRead> PhysicalMemoryRead for T {
    fn try_read_bytes_physical_into(&self, physical_address: u64, buffer: &mut [u8]) -> Option<()> {
        unsafe {
            let map = self.map_io_space(physical_address, buffer.len())?;
            self.try_read_bytes_into(map, buffer)?;
            self.unmap_io_space(map, buffer.len())?;
            Some(())
        }
    }
}

impl<T: MapPhysical + KernelMemoryWrite> PhysicalMemoryWrite for T {
    fn try_write_bytes_physical(&self, physical_address: u64, buffer: &[u8]) -> Option<()> {
        unsafe {
            let map = self.map_io_space(physical_address, buffer.len())?;
            self.try_write_bytes(map, buffer)?;
            self.unmap_io_space(map, buffer.len())?;
            Some(())
        }
    }
}

pub trait TranslatePhysical {
    fn physical_address(&self, virtual_address: u64) -> Option<u64>;
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

impl<'a, T: MapPhysical + KernelMemoryRead + KernelMemoryWrite> crate::MemoryRead
for MappedPhysicalMemory<'a, T>
{
    fn try_read_bytes_into(&self, address: u64, buffer: &mut [u8]) -> Option<()> {
        if address + buffer.len() as u64 > self.base + self.size as u64 {
            return None;
        }
        self.api
            .try_read_bytes_into(self.base as u64 + address, buffer)
    }
}

impl<'a, T: MapPhysical + KernelMemoryRead + KernelMemoryWrite> crate::MemoryWrite
for MappedPhysicalMemory<'a, T>
{
    fn try_write_bytes(&self, address: u64, buffer: &[u8]) -> Option<()> {
        if address + buffer.len() as u64 > self.base + self.size as u64 {
            return None;
        }
        self.api.try_write_bytes(self.base as u64 + address, buffer)
    }
}

impl<'a, T: MapPhysical + KernelMemoryRead + KernelMemoryWrite> Drop
for MappedPhysicalMemory<'a, T>
{
    fn drop(&mut self) {
        unsafe { self.api.unmap_io_space(self.base, self.size).unwrap() }
    }
}
