#[tarpc::service]
pub trait SystemHandle {
    // Gets a down or up state of a certain key using a VK key code:
    // https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
    async fn get_key_state(key: i32) -> bool;
}

