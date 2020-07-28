use tokio::net::TcpStream;
use tarpc::client;
use tarpc::serde_transport::Transport;

mod util;
mod functions;

pub use functions::*;

static mut CONNECTION: Option<system_host::SystemHandleClient> = None;

#[cfg(not(windows))]
/// If we're not on windows, we want to connect via a socket address
pub async fn connect(address: &std::net::SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let transport = tarpc::serde_transport::tcp::connect(&address, tokio_serde::formats::Json::default()).await?;
    let client = system_host::SystemHandleClient::new(client::Config::default(), transport).spawn()?;
    unsafe { CONNECTION = Some(client) }
    Ok(())
}

#[cfg(windows)]
/// If we're on windows, we want to connect via a channel
pub fn connect() {
    unimplemented!()
}

/// Returns the RPC connection
fn get_connection() -> &'static mut system_host::SystemHandleClient {
    unsafe {
        match &mut CONNECTION {
            Some(connection) => connection,
            None => panic!("Attempted to run a system function without initializing the system first"),
        }
    }
}