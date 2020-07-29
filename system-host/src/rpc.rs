use log::*;
use futures::{future, StreamExt};
use tarpc::server;
use tarpc::server::{Handler, Serve};
use tarpc::context::Context;
use crate::system;
use crate::service_definition::*;


#[derive(Clone)]
struct SystemHandleServer;

#[tarpc::server]
impl SystemHandle for SystemHandleServer {
    async fn get_key_state(self, _: Context, key: i32) -> bool {
        unsafe { system::get_key_state(key) }
    }

    async fn move_mouse_relative(self, _: Context, dx: i32, dy: i32) {
        unsafe { system::move_mouse_relative(dx, dy) }
    }
}

pub async fn listen(address: &std::net::SocketAddr) -> std::io::Result<()> {
    debug!("Attempting to listen on {}", &address);
    let transport = tarpc::serde_transport::tcp::listen(&address, tokio_serde::formats::Json::default)
        .await?
        .filter_map(|r| future::ready(r.ok()));

    info!("RPC listening on {}", &address);

    server::new(server::Config::default())
        .incoming(transport)
        .respond_with(SystemHandleServer.serve())
        .await;
    Ok(())
}
