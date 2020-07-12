use crate::config::Config;
use crate::sdk::FortniteContext;
use crate::CHEAT_TICKRATE;

use log::*;
use memlib::memory::{Address, Pointer};
use memlib::util::{to_hex_string, LoopTimer};
use std::error::Error;

// The main loop of the cheat
// Returns an error if there is an error with any of the tick functions
pub fn hack_loop(ctx: FortniteContext) -> Result<(), Box<dyn Error>> {
    let config = Config::default();

    let mut timer = LoopTimer::new(CHEAT_TICKRATE);

    loop {
        timer.wait();
        let uworld_ptr: Pointer<Address> = ctx.handle.read_memory(ctx.base_address + 0x8C750C0);
        debug!("{}", uworld_ptr);
    }

    Ok(())
}
