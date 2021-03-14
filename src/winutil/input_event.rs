use std::collections::HashMap;
use std::sync::{Arc, mpsc, Mutex, Once};
use std::sync::mpsc::{Receiver, Sender, SendError};

use lazy_static::lazy_static;
use log::*;
pub use win_key_codes::*;
use winapi::um::winuser::GetKeyboardState;

use crate::winutil::get_keyboard_state;
use std::time::Duration;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Event {
    KeyDown(i32),
    KeyUp(i32),
}

/// Listens to Windows input events and sends them to a channel
pub struct InputEventListener {
    pub recv: Receiver<Event>
}

impl InputEventListener {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let (send, recv) = mpsc::channel();

        // Only make one thread listening for key events
        THREAD_START.call_once(|| {
            std::thread::spawn(|| {
                listen_thread();
            });
        });

        // Add the sender so we can receive
        SENDERS.lock().unwrap().push(send);
        Self { recv }
    }

    pub fn is_key_down(&self, key: i32) -> bool {
        KEYS_DOWN.lock().unwrap().contains(&key)
    }

    pub fn get_keys_down(&self) -> Vec<i32> {
        KEYS_DOWN.lock().unwrap().clone()
    }

    /// Handles all events using the handler callback indefinitely
    pub fn handle_events_blocking(&self, handler: impl Fn(Event)) {
        self.recv.iter().for_each(handler)
    }

    /// Handles all events queued and doesn't block
    pub fn handle_events(&self, handler: impl Fn(Event)) {
        loop {
            match self.recv.try_recv() {
                Ok(event) => handler(event),
                Err(_) => return
            }
        }
    }
}

/// Iterates through every event in the queue until there are none left
impl Iterator for &InputEventListener {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.recv.try_recv().ok()
    }
}

lazy_static! {
    // The listen thread will send events to every sender in this array,
    static ref SENDERS: Mutex<Vec<Sender<Event>>> = Mutex::new(vec![]);
    static ref KEYS_DOWN: Mutex<Vec<i32>> = Mutex::new(vec![]);
}

static THREAD_START: Once = Once::new();

fn listen_thread() {
    let mut keyboard_state = get_keyboard_state();
    let mut prev_keyboard_state = keyboard_state.clone();
    loop {
        std::thread::sleep(Duration::from_millis(1));

        keyboard_state = get_keyboard_state();

        // Update global list of keys down
        *KEYS_DOWN.lock().unwrap() = keyboard_state.iter()
            .filter(|(_, &down)| down)
            .map(|(&key, _)| key)
            .collect();

        let mut events = Vec::new();
        for (key, prev_down) in prev_keyboard_state {
            let down = *keyboard_state.get(&key).unwrap();
            // If the state before is the same, continue
            if prev_down == down {
                continue;
            }

            // If the keys are different, add the event
            events.push(match down {
                true => Event::KeyDown(key),
                false => Event::KeyUp(key)
            });
        }

        // For every sender, send all of the events. If there is an error sending the event,
        // remove it from the senders
        SENDERS.lock().unwrap()
            .retain(|sender| {
                let mut retain = true;
                // Send every event
                for event in &events {
                    let result = sender.send(event.clone());
                    if let Err(e) = result {
                        debug!("Sending event to sender returned error: {}. Removing sender", e);
                        retain = false;
                        break;
                    }
                }
                retain
            });

        prev_keyboard_state = keyboard_state
    }
}