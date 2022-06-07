use std::cell::RefCell;
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