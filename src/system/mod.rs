// This module allows for interactions with the system itself.
// If this application is running on the host, it will interact with
// the host directly. If the game is running on a KVM, it will create
// an RPC connection to the guest using the system-host package

/// A handle to the system.
/// Either runs locally or runs remotely using an RPC
/// server hosted on the client.
/// Regardless on the communication medium, the implementation
/// for this is inside the system-host crate
pub trait SystemHandle {
    fn get_key_state(key: i32) -> bool;
}