// As a library, we only want to export the system handle interface definition
pub mod service_definition;
pub use service_definition::*;

#[cfg(windows)]
pub mod rpc;
#[cfg(windows)]
pub mod system;
