use std::cell::{Cell};
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

impl<T: AsRef<[u8]>> MemoryRead for Cell<T> {
    fn try_read_bytes_into(&self, address: u64, buffer: &mut [u8]) -> Option<()> {
        let self_buf = unsafe { self.as_ptr().as_ref() }.unwrap();
        if address as usize + buffer.len() > self_buf.as_ref().len() {
            return None;
        }

        buffer.copy_from_slice(&self_buf.as_ref()[address as usize..address as usize + buffer.len()]);

        Some(())
    }
}

impl<T: AsMut<[u8]>> MemoryWrite for Cell<T> {
    fn try_write_bytes(&self, address: u64, buffer: &[u8]) -> Option<()> {
        let self_buf = unsafe { self.as_ptr().as_mut() }.unwrap();
        if address as usize + buffer.len() > self_buf.as_mut().len() {
            return None;
        }

        self_buf.as_mut()[address as usize..address as usize + buffer.len()].copy_from_slice(buffer);

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
    fn test_cell_read() {
        let mut buffer = [0u8; 10];
        let cell = Cell::new(buffer);
        use_read(cell);
    }

    #[test]
    fn test_cell_write() {
        let mut buffer = [0u8; 10];
        let cell = Cell::new(buffer);
        use_write(cell);
    }

    #[test]
    fn test_cell_readwrite() {
        let mut buffer = [0u8; 10];
        let cell = Cell::new(buffer);
        use_readwrite(cell);
    }

    static mut TEST_BUF: [u8; 10] = [0u8; 10];
    fn get_test_buf() -> &'static mut [u8] {
        unsafe {
            &mut TEST_BUF
        }
    }

    #[test]
    fn test_static_cell_read() {
        let cell = Cell::new(get_test_buf());
        use_read(cell);
    }

    #[test]
    fn test_static_cell_write() {
        let cell = Cell::new(get_test_buf());
        use_write(cell);
    }

    #[test]
    fn test_static_cell_readwrite() {
        let cell = Cell::new(get_test_buf());
        use_readwrite(cell);
    }
}