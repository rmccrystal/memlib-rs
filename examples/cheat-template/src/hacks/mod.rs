use crate::config::Config;
use crate::sdk::*;

use log::*;
use memlib::util::LoopTimer;

// The main loop of the cheat
// Returns an error if there is an error with any of the tick functions
pub fn hack_loop(game: Game) -> Result<(), Box<dyn std::error::Error>> {
    // Use the default config. We can change this later to load from a file
    let mut config = Config::default();

    // Create a timer from the tickrate of the cheat
    let mut timer = LoopTimer::new(crate::CHEAT_TICKRATE);

    loop {
        // Run the loop `CHEAT_TICKRATE` times per second
        timer.wait();

        // (main cheat code)
    }
}
