use crate::sdk::FortniteContext;

use memlib::logger::MinimalLogger;
use memlib::memory;

use log::*;
use std::error::Error;

mod config;
mod hacks;
mod sdk;

pub const PROCESS_NAME: &str = "csgo.exe";
pub const CHEAT_TICKRATE: u64 = 1;

const LOG_LEVEL: LevelFilter = LevelFilter::Debug;

fn run() -> Result<(), Box<dyn Error>> {
    MinimalLogger::init(LOG_LEVEL)?;

    let handle = memory::Handle::new(PROCESS_NAME)?;
    let ctx = FortniteContext::new(handle);

    hacks::hack_loop(ctx)?;

    Ok(())
}

fn main() {
    std::process::exit(match run() {
        Ok(_) => 0,
        Err(err) => {
            error!("{}", err);
            1
        }
    })
}
