use crate::math::Vector2;
use log::*;
use pretty_hex::*;
use std::thread::sleep;
use std::time::{Duration, Instant};
use parking_lot::{RwLockWriteGuard, RwLockReadGuard};
use parking_lot::lock_api::RawRwLock;
use std::ops::Deref;
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

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
        trace!(
            "Loop took {} ms",
            (Instant::now() - self.last_tick).as_millis()
        );

        // Print if the loop took too long
        let mut had_to_sleep = false;
        // Wait until we've waited enough time
        while self.last_tick + Duration::from_millis(self.ms_delay) >= Instant::now() {
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

/// Gets a bounding box of a player. Takes a list of points on the screen (normally bones)
/// and returns the smallest bounding box that includes all the points from bottom left to top right.
/// Note that directions are right then down
pub fn get_boudning_box(points: Vec<Vector2>) -> (Vector2, Vector2) {
    let mut left: f32 = points[0].x;
    let mut right: f32 = points[0].x;
    let mut top: f32 = points[0].y;
    let mut bottom: f32 = points[0].y;

    for point in points {
        if point.x < left {
            left = point.x
        }
        if point.x > right {
            right = point.x
        }

        if point.y < top {
            top = point.y
        }
        if point.y > bottom {
            bottom = point.y
        }
    }

    (Vector2 { x: left, y: bottom }, Vector2 { x: right, y: top })
}

pub struct InitCell<T> {
    inner: UnsafeCell<Option<parking_lot::RwLock<T>>>,
    init: AtomicBool,
}

unsafe impl<T> Sync for InitCell<T> {}

impl<T: Clone> InitCell<T> {
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(None),
            init: AtomicBool::new(false),
        }
    }

    pub fn init(&self, val: T) {
        if self.init.load(Ordering::Relaxed) {
            panic!("InitCell init() called twice");
        }
        let inner = self.inner.get();
        unsafe { *inner = Some(parking_lot::RwLock::new(val)); }
        self.init.store(false, Ordering::Relaxed);
    }

    pub fn get_clone(&self) -> T {
        (*self.get_ref()).clone()
    }

    fn get_inner(&self) -> &parking_lot::RwLock<T> {
        unsafe { self.inner.get().as_ref() }.unwrap().as_ref().unwrap()
    }

    /// Gets a refrence of the contents of the cell
    pub fn get_ref(&self) -> RwLockReadGuard<'_, T> {
        self.get_inner().read()
    }

    /// Gets a mutable refrence of the contents of the cell
    pub fn get_mut(&self) -> RwLockWriteGuard<'_, T> {
        self.get_inner().write()
    }

    /// Sets the value
    pub fn set(&self, value: T) {
        // Wait until cell is writable
        let mut lock = self.get_inner().write();
        *lock = value;
    }
}

impl<T: Copy> InitCell<T> {
    /// Gets a copy of whatever is stored in the cell
    pub fn get(&self) -> T {
        *self.get_inner().read()
    }
}

pub struct GlobalCell<T> {
    inner: parking_lot::RwLock<T>
}

impl<T: Clone> GlobalCell<T> {
    pub const fn new(value: T) -> Self {
        Self {
            inner: parking_lot::const_rwlock(value)
        }
    }

    /// Gets a refrence of the contents of the cell
    pub fn get_ref(&self) -> RwLockReadGuard<'_, T> {
        self.inner.read()
    }

    /// Gets a mutable refrence of the contents of the cell
    pub fn get_mut(&self) -> RwLockWriteGuard<'_, T> {
        self.inner.write()
    }

    pub fn get_clone(&self) -> T {
        (*self.get_ref()).clone()
    }

    /// Sets the value
    pub fn set(&self, value: T) {
        // Wait until cell is writable
        let mut lock = self.inner.write();
        *lock = value;
    }
}

impl<T: Copy> GlobalCell<T> {
    /// Gets a copy of whatever is stored in the cell
    pub fn get(&self) -> T {
        *self.inner.read()
    }
}
