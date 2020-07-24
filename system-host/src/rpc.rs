use jsonrpc_derive::rpc;
use jsonrpc_core::{Result, IoHandler};
use jsonrpc_tcp_server::ServerBuilder;
use std::net::{SocketAddr, SocketAddrV4};
use log::*;

#[rpc(server)]
pub trait SystemHandleInterface {
    // Gets a down or up state of a certain key using a VK key code:
    // https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
    #[rpc(name = "get_key_state")]
    fn get_key_state(&self, key: i32) -> Result<bool>;
}

pub struct SystemHandle;

impl SystemHandleInterface for SystemHandle {
    fn get_key_state(&self, key: i32) -> Result<bool> {
        Ok(true)
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