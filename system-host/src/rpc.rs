use crate::service_definition::*;
use crate::system;
use futures::{future, StreamExt};
use log::*;
use tarpc::context::Context;
use tarpc::server::{Handler};
use tarpc::{client, server};
use tokio::stream;

#[derive(Clone)]
struct SystemHandleServer;

#[tarpc::server]
impl SystemHandle for SystemHandleServer {
    async fn get_key_state(self, _: Context, key: i32) -> bool {
        system::get_key_state(key)
    }

    async fn move_mouse_relative(self, _: Context, dx: i32, dy: i32) {
        system::move_mouse_relative(dx, dy)
    }
}

pub async fn listen(address: &std::net::SocketAddr) -> std::io::Result<()> {
    debug!("Attempting to listen on {}", &address);
    let transport =
        tarpc::serde_transport::tcp::listen(&address, tokio_serde::formats::Json::default)
            .await?
            .filter_map(|r| future::ready(r.ok()));

    info!("RPC listening on {}", &address);

    server::new(server::Config::default())
        .incoming(transport)
        .respond_with(SystemHandleServer.serve())
        .await;
    Ok(())
}

pub async fn listen_channel() -> Result<SystemHandleClient, Box<dyn std::error::Error>> {
    let (client_transport, server_transport) = tarpc::transport::channel::unbounded();

    let server = server::new(server::Config::default())
        .incoming(stream::once(server_transport))
        .respond_with(SystemHandleServer.serve());

    // Spawn the server
    tokio::spawn(server);

    // Connect through the channel
    let client = SystemHandleClient::new(client::Config::default(), client_transport).spawn()?;

    Ok(client)
}
