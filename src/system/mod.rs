use tokio::net::TcpStream;
use tarpc::client;
use tarpc::serde_transport::Transport;

static mut CONNECTION: Option<system_host::SystemHandleClient> = None;
static mut TOKIO_RUNTIME: Option<tokio::runtime::Runtime> = None;

#[cfg(not(windows))]
/// If we're not on windows, we want to connect via a socket address
pub fn connect(address: &std::net::SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    run_async(async move {
        let transport = tarpc::serde_transport::tcp::connect(&address, tokio_serde::formats::Json::default).await?;
        let mut client = system_host::SystemHandleClient::new(client::Config::default(), transport).spawn()?;
        Ok(())
    })
}

#[cfg(windows)]
/// If we're on windows, we want to connect via a channel
pub fn connect() {
    unimplemented!()
}

/// Runs an async function from a non asyncronous context
fn run_async<T>(future: impl std::future::Future<Output=T>) -> T {
    // Either get a preexisting runtime or create a new one
    let mut runtime = unsafe {
        match &TOKIO_RUNTIME {
            Some(runtime) => runtime,
            None => {
                let new_runtime = tokio::runtime::Runtime::new().unwrap();
                TOKIO_RUNTIME = Some(new_runtime);
                &new_runtime
            }
        }
    };

    runtime.block_on(future)
}


pub fn get_system() -> &'static system_host::SystemHandleClient {
    unsafe { &CONNECTION.unwrap() }
}