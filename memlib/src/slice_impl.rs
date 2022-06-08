use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};
use super::*;

impl<'a> MemoryRead for &'a [u8] {
    fn try_read_bytes_into(&self, address: u64, buffer: &mut [u8]) -> Option<()> {
        if address as usize + buffer.len() > self.len() {
            return None;
        }

        buffer.copy_from_slice(&self[address as usize..address as usize + buffer.len()]);

        Some(())
    }
}

impl MemoryRead for RefCell<[u8]> {
    fn try_read_bytes_into(&self, address: u64, buffer: &mut [u8]) -> Option<()> {
        if address as usize + buffer.len() > self.borrow().len() {
            return None;
        }

        buffer.copy_from_slice(&self.borrow()[address as usize..address as usize + buffer.len()]);

        Some(())
    }
}

impl MemoryWrite for RefCell<[u8]> {
    fn try_write_bytes(&self, address: u64, buffer: &[u8]) -> Option<()> {
        if address as usize + buffer.len() > self.borrow().len() {
            return None;
        }

        self.borrow_mut()[address as usize..address as usize + buffer.len()].copy_from_slice(buffer);

        Some(())
    }
}

impl<T: AsRef<[u8]> + Copy> MemoryRead for Cell<T> {
    fn try_read_bytes_into(&self, address: u64, buffer: &mut [u8]) -> Option<()> {
        if address as usize + buffer.len() > self.get().as_ref().len() {
            return None;
        }

        buffer.copy_from_slice(&self.get().as_ref()[address as usize..address as usize + buffer.len()]);

        Some(())
    }
}

impl<T: AsMut<[u8]> + Copy> MemoryWrite for Cell<T> {
    fn try_write_bytes(&self, address: u64, buffer: &[u8]) -> Option<()> {
        if address as usize + buffer.len() > self.get().as_mut().len() {
            return None;
        }

        self.get().as_mut()[address as usize..address as usize + buffer.len()].copy_from_slice(buffer);

        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn use_read(read: impl MemoryRead) {}
    fn use_write(write: impl MemoryWrite) {}
    fn use_readwrite(readwrite: impl MemoryRead + MemoryWrite) {}

    #[test]
    fn test_slice_read() {
        let mut buffer = [0u8; 10];
        let slice = &buffer[..];
        use_read(slice);
    }

    #[test]
    fn test_refcell_read() {
        let mut buffer = [0u8; 10];
        let cell = Cell::new(buffer);
        use_read(cell);
    }

    #[test]
    fn test_refcell_write() {
        let mut buffer = [0u8; 10];
        let cell = Cell::new(buffer);
        use_write(cell);
    }

    #[test]
    fn test_refcell_readwrite() {
        let mut buffer = [0u8; 10];
        let cell = Cell::new(buffer);
        use_readwrite(cell);
    }
}