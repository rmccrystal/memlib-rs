# memlib

Memlib is an abstraction layer for dealing with raw memory buffers, especially when dealing with process memory.
It contains many traits that can be used across crates to have a common interface for dealing with memory.
The core traits of this library are `MemoryRead` and `MemoryWrite` which contain extensive utility functions
in the form of extension traits `MemoryReadExt` and `MemoryWriteExt`. This crate also contains other traits dealing
with memory, including or `ModuleList`, `MemoryProtect`, `MemoryAllocate`. There are many more traits available
for dealing with utility functions and overlay rendering.
