use super::*;
use super::util::run_async;
use tarpc::context;

// Gets a down or up state of a certain key using a VK key code:
// https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
pub fn get_key_state(key: i32) -> bool {
    run_async(get_connection().get_key_state(context::current(), key)).unwrap()
}