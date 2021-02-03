use std::collections::HashMap;

use winapi::um::winuser::{GetAsyncKeyState, GetKeyboardState, GetKeyState};

/// Returns a list of all keys and weather or not they are down
pub(crate) fn get_keyboard_state() -> HashMap<i32, bool> {
    /*
    let mut keys = vec![0u8; 256];
    let _ = unsafe { GetKeyState(0) };
    let success = unsafe { GetKeyboardState(keys.as_mut_ptr()) };
    if success == 0 {
        panic!(std::io::Error::last_os_error());
    }

    keys.into_iter()
        .enumerate()
        .map(|state| (state.0 as i32, state.1 & 1 == 1))
        .collect()
     */
    const NUM_KEYS: usize = 254;

    let mut keys = HashMap::with_capacity(NUM_KEYS);
    for i in 0i32..=(NUM_KEYS as i32) {
        keys.insert(i, is_key_down(i));
    }

    keys
}

pub fn is_key_down(key: i32) -> bool {
    let state = unsafe { GetAsyncKeyState(key as _) } as u16;
    (state & 0x8000) != 0
}
