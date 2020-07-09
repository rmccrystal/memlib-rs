use std::time::{Instant, Duration};
use std::thread::sleep;

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
        // Wait until we've waited enough time
        while self.last_tick + Duration::from_millis(self.ms_delay) > Instant::now() {
            sleep(Duration::from_micros(1));
        }
        // Update last tick
        self.last_tick = Instant::now();
    }
}