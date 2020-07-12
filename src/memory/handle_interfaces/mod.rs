#[cfg(target_os = "linux")]
pub mod kvm_handle;

#[cfg(target_os = "windows")]
pub mod winapi_handle;
