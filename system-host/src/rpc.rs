use std::net::{SocketAddr, SocketAddrV4};
use log::*;
use tarpc::context::Context;

#[tarpc::service]
pub trait SystemHandle {
    // Gets a down or up state of a certain key using a VK key code:
    // https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
    fn get_key_state(key: i32) -> bool;
}

pub struct SystemHandleServer;

impl SystemHandle for SystemHandleServer {
    fn get_key_state(self, ctx: Context, key: i32) -> bool {
        true
    }
}

pub fn listen(address: &std::net::SocketAddr) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut io = IoHandler::new();
    let rpc = SystemHandle;

    io.extend_with(rpc.to_delegate());

    debug!("RPC attempting to listen on {}", address);

    let server = ServerBuilder::new(io)
        .start(&address)?;

    info!("RPC server listening on {}", address);

    server.wait();

    Ok(())
}