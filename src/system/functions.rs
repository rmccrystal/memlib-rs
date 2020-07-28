use super::*;
use super::util::run_async;
use tarpc::context;

// Gets a down or up state of a certain key using a VK key code:
// https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
pub async fn get_key_state(key: i32) -> bool {
    get_connection().get_key_state(context::current(), key).await.unwrap()
}