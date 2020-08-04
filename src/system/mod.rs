use tarpc::client;

use log::*;

mod functions;
mod util;

use crate::system::util::run_async;
pub use functions::*;

static mut CONNECTION: Option<system_host::SystemHandleClient> = None;

#[cfg(not(windows))]
/// If we're not on windows, we want to connect via a socket address
pub fn connect(address: &std::net::SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    run_async(async move {
        let transport =
            tarpc::serde_transport::tcp::connect(&address, tokio_serde::formats::Json::default())
                .await?;
        let client =
            system_host::SystemHandleClient::new(client::Config::default(), transport).spawn()?;
        unsafe { CONNECTION = Some(client) }
        info!("Connected to system {}", &address);
        Ok(())
    })
}

#[cfg(windows)]
/// If we're on windows, we want to connect via a channel
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    run_async(async move {
        // Start the server over a channel
        use futures::future;
        use tarpc::server;
        use tarpc::server::{Handler, Serve};
        use tokio::stream;

        let (client_transport, server_transport) = tarpc::transport::channel::unbounded();
        let server = server::new(server::Config::default())
            .incoming(server_transport)
            .respond_with(system_host::rpc::SystemHandleServer.serve())

        // Spawn the server
        tokio::spawn(server);

        // Connect through the channel
        let client =
            system_host::SystemHandleClient::new(client::Config::default(), client_transport)
                .spawn()?;
        unsafe { CONNECTION = Some(client) }

        Ok(())
    })
}

/// Returns the RPC connection
fn get_connection() -> &'static mut system_host::SystemHandleClient {
    unsafe {
        match &mut CONNECTION {
            Some(connection) => connection,
            None => {
                panic!("Attempted to run a system function without initializing the system first")
            }
        }
    }
}
