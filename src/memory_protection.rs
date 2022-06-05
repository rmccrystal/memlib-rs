use bitflags::bitflags;

bitflags! {
    pub struct MemoryProtection: u32 {
        const NONE = 0x0;
        /// Enables execute access to the committed region of pages.An attempt to write to the committed region results in an access violation.
        const EXECUTE = 0x10;
        /// Enables read access to the committed region of pages. An attempt to write to the committed region results in an access violation.
        const EXECUTE_READ = 0x20;
        /// Enables read and execute access to the committed region of pages. An attempt to write to the committed region results in an access violation.
        const EXECUTE_READWRITE = 0x40;
        /// Enables read, write, and execute access to the committed region of pages.
        const EXECUTE_WRITECOPY = 0x80;
        /// Disables all access to the committed region of pages. An attempt to read from, write to, or execute the committed region results in an access violation.
        const NOACCESS = 0x01;
        /// Enables read access to the committed region of pages. An attempt to write to the committed region results in an access violation.
        const READONLY = 0x02;
        /// Enables read and write access to the committed region of pages.
        const READWRITE = 0x04;
        /// Enables read-only or copy-on-write access to a mapped view of a file mapping object. An attempt to write to a committed copy-on-write page results in a private copy of the page being made for the process.
        const WRITECOPY = 0x08;
        /// Pages in the region become guard pages. Any attempt to access a guard page causes the system to raise a STATUS_GUARD_PAGE_VIOLATION exception and turn off the guard page status. Guard pages thus act as a one-time access alarm.
        const GUARD = 0x100;
        /// Sets all pages to be non-cachable. Applications should not use this attribute except when explicitly required for a device.
        const NOCACHE = 0x200;
        /// Sets all pages to be write-combined.
        const WRITECOMBINE = 0x400;
    }
}