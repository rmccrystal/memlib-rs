use log::*;

mod functions;
mod util;

use crate::system::util::run_async;
pub use functions::*;

pub use win_key_codes as keys;
use tarpc::client;

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
    let client = run_async(system_host::rpc::listen_channel())?;

    unsafe { CONNECTION = Some(client) }
    info!("Connected to system through channel");

    Ok(())
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
