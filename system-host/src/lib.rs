// Re exporting everything in the system_rpc module in case we want
// to import this crate and use it as a host

mod rpc;
pub use rpc::{SystemHandle, SystemHandleInterface};

