# memlib-rs
An external game hacking library for Rust. This library allows you to read and write memory to a process through the following means:
* Modifying memory on a Windows KVM with a Linux host using DMA with [vmread-rs](https://github.com/h33p/vmread-rs)
* Directly calling ReadProcessMemory & WriteProcessMemory from the WinAPI
* Using a driver written in rust to read / write memory from kernel mode

Additionally, this library includes several utility classes which most cheats will use.
