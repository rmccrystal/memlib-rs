use log::*;

pub mod rpc;
pub mod service_definition;
pub mod system;

pub const LOG_LEVEL: LevelFilter = LevelFilter::Debug;
pub const LISTEN_IP: &str = "0.0.0.0";
pub const LISTEN_PORT: u16 = 9800;

#[tokio::main]
async fn main() {
    // Init logger
    simplelog::TermLogger::init(
        LOG_LEVEL,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
    )
    .unwrap();

    unsafe {
        system::move_mouse_relative(100, 10);
    }

    // Create listen address
    let listen_addr: std::net::SocketAddr = format!("{}:{}", LISTEN_IP, LISTEN_PORT)
        .parse()
        .expect("Invalid listen IP or port");

    // Start listening
    if let Err(error) = rpc::listen(&listen_addr).await {
        error!("{}", error);
    }
}
