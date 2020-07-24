use jsonrpc_derive::rpc;
use jsonrpc_tcp_server::*;
use jsonrpc_tcp_server::jsonrpc_core::*;

#[rpc]
pub trait SystemHandle {
    // Gets a down or up state of a certain key using a VK key code:
    // https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
    #[rpc(name = "get_key_state")]
    fn get_key_state(&self, key: i32) -> bool;
}

struct SystemHandleImpl;

impl SystemHandle for SystemHandleImpl {
    fn get_key_state(&self, key: i32) -> bool {
        true
    }
}

pub fn listen(address: impl Into<std::net::SocketAddr>, port: u16) {
    let mut io = IoHandler::default();
}