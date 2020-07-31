use pretty_hex::*;
use std::thread::sleep;
use std::time::{Duration, Instant};
use log::*;

/// A timer which runs a loop at a consistent rate
/// For example, in a game hack, we might want to run the main
/// loop with a certain tickrate. To accomplish that, we would
/// do the following:
/// ```
/// use memlib::util::LoopTimer;
///
/// let mut timer = LoopTimer::new(CHEAT_TICKRATE);
///
/// loop {
///     timer.wait();
///
///     // This will run 60 times per second unless it takes too much time,
///     // in which case it will run as fast as possible until it catches up
/// }
/// ```
pub struct LoopTimer {
    pub tickrate: u64,
    pub last_tick: Instant,
    ms_delay: u64,
}

impl LoopTimer {
    pub fn new(tickrate: u64) -> Self {
        Self {
            tickrate,
            last_tick: Instant::now(),
            ms_delay: 1000 / tickrate,
        }
    }

    /// Waits until we have waited enough time since the `last_tick` according to the `tickrate`
    pub fn wait(&mut self) {
        trace!("Loop took {} ms", (Instant::now() - self.last_tick).as_millis());

        // Print if the loop took too long
        let mut had_to_sleep = false;
        // Wait until we've waited enough time
        while self.last_tick + Duration::from_millis(self.ms_delay) > Instant::now() {
            sleep(Duration::from_micros(1));
            had_to_sleep = true;
        }
        if !had_to_sleep {
            let ms_diff = (Instant::now() - self.last_tick).as_millis();
                warn!("Loop took {} ms too long (delay: {}ms, took: {}ms)",
                      ms_diff - self.ms_delay as u128,
                      self.ms_delay,
                      ms_diff);
        }

        // Update last tick
        self.last_tick = Instant::now();
    }
}

pub fn to_hex_string(buf: &[u8]) -> String {
    buf.hex_dump().to_string()
}
