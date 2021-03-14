// From https://github.com/ekknod/logitech-cve

use log::*;
use winapi::um::winnt::{HANDLE, PCWSTR};
use std::mem::MaybeUninit;
use ntapi::ntrtl::RtlInitUnicodeString;
use winapi::shared::ntdef::InitializeObjectAttributes;
use std::ptr::null_mut;
use ntapi::ntioapi::{NtCreateFile, IO_STATUS_BLOCK, FILE_NON_DIRECTORY_FILE, FILE_SYNCHRONOUS_IO_NONALERT, NtDeviceIoControlFile};
use ntapi::winapi::um::winnt::*;
use anyhow::*;
use super::ToError;
use ntapi::ntzwapi::ZwClose;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct MouseIO {
    pub button: i8,
    pub x: i8,
    pub y: i8,
    pub wheel: i8,
    pub unk1: i8,
}

pub enum MouseInput {
    LeftDown,
    RightDown,
    MiddleDown,
    Up,
    Move(i8, i8),
    Wheel(i8),
    _Button(i8)
}

pub struct LogitechDriver {
    handle: HANDLE,
    io: IO_STATUS_BLOCK,
}

unsafe impl Send for LogitechDriver {}
unsafe impl Sync for LogitechDriver {}

impl LogitechDriver {
    pub fn new() -> Result<Self> {
        let (handle, io) = unsafe { Self::init()? };

        Ok(Self { handle, io })
    }

    pub fn send_input(&mut self, input: &MouseInput) {
        use std::mem::zeroed;
        let io = unsafe {
            match input {
                MouseInput::LeftDown => MouseIO { button: 1, ..zeroed() },
                MouseInput::RightDown => MouseIO { button: 2, ..zeroed() },
                MouseInput::MiddleDown => MouseIO { button: 3, ..zeroed() },
                MouseInput::Up => MouseIO { button: 0, ..zeroed() },
                MouseInput::Move(x, y) => MouseIO { x: *x, y: *y, ..zeroed() },
                MouseInput::Wheel(delta) => MouseIO {wheel: *delta, ..zeroed()},
                MouseInput::_Button(button) => MouseIO { button: *button, ..zeroed() }
            }
        };

        unsafe {
            if let Err(e) = self.call_mouse(&io) {
                error!("Logitech mouse CVE encountered error: {:?}, recreating device", e);
                let (handle, io) = Self::init().unwrap();
                self.handle = handle;
                self.io = io;
            }
        };
    }

    unsafe fn call_mouse(&self, buffer: &MouseIO) -> Result<()> {
        trace!("Sending mouse buffer to Logitech CVE: {:?}", buffer);
        let mut block = MaybeUninit::uninit().assume_init();
        NtDeviceIoControlFile(
            self.handle,
            0 as _,
            None,
            0 as _,
            &mut block,
            0x2a2010,
            buffer as *const _ as _,
            std::mem::size_of::<MouseIO>() as _,
            0 as _,
            0 as _,
        ).to_err()
    }

    unsafe fn init() -> Result<(HANDLE, IO_STATUS_BLOCK)> {
        let device_name = c_string_w!("\\??\\ROOT#SYSTEM#0002#{1abc05c0-c378-41b9-9cef-df1aba82b015}");
        Ok(match Self::device_initialize(device_name.as_ptr()) {
            Ok(n) => n,
            Err(err) => {
                let device_name = c_string_w!("\\??\\ROOT#SYSTEM#0001#{1abc05c0-c378-41b9-9cef-df1aba82b015}");
                match Self::device_initialize(device_name.as_ptr()) {
                    Ok(n) => n,
                    Err(e) => bail!(e)
                }
            }
        })
    }

    unsafe fn device_initialize(device_name: PCWSTR) -> Result<(HANDLE, IO_STATUS_BLOCK)> {
        let mut name = MaybeUninit::uninit().assume_init();
        let mut attr = MaybeUninit::uninit().assume_init();

        RtlInitUnicodeString(&mut name, device_name);
        InitializeObjectAttributes(&mut attr, &mut name, 0, null_mut(), null_mut());

        let mut handle = MaybeUninit::uninit().assume_init();
        let mut io = MaybeUninit::uninit().assume_init();

        let status = NtCreateFile(
            &mut handle,
            GENERIC_WRITE | SYNCHRONIZE,
            &mut attr,
            &mut io,
            0 as _,
            FILE_ATTRIBUTE_NORMAL,
            0,
            3,
            FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
            0 as _,
            0,
        ).to_err()?;

        Ok((handle, io))
    }
}

impl Drop for LogitechDriver {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { ZwClose(self.handle); }
        }
    }
}