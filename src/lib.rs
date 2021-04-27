#![feature(const_fn)]

#[macro_use]
pub mod macros;
#[macro_use]

pub mod hacks;
pub mod logger;
pub mod math;
pub mod memory;
pub mod util;

#[cfg(feature = "internal")]
pub mod internal;
