/// A macro for making a getter member function using a base address and an offset
/// The syntax is:
/// ```
/// memory_getter!(function_name, return_type, offset, base_address);
/// ```
///
/// Example:
/// ```
/// memory_getter!(get_health, i32, netvars::m_iHealth, self.base);
/// // ...
/// assert!(get_health() > 0);
/// ```
#[macro_export]
macro_rules! memory_getter {
    ($func_name: ident, String, $offset:expr,  $base_address:expr) => {
        pub fn $func_name(&self) -> String {
            memlib::memory::read_string($base_address + $offset)
        }
    };
    ($func_name: ident, $return_type:ty, $offset:expr, $base_address:expr) => {
        pub fn $func_name(&self) -> $return_type {
            memlib::memory::read_memory($base_address + $offset)
        }
    };
}

/// A macro similar to memory_getter except it creates a setter function
/// The syntax is:
/// ```
/// memory_setter!(function_name, param_type, offset, base_address);
/// ```
///
/// Example:
/// ```
/// memory_setter!(set_health, i32, netvars::m_iHealth, self.base);
/// // ...
/// player.set_health(50);
/// ```
#[macro_export]
macro_rules! memory_setter {
    ($func_name: ident, $param_type:ty, $offset:expr, $base_address:expr) => {
        pub fn $func_name(&self, value: $param_type) {
            memlib::memory::write_memory($base_address + $offset, value)
        }
    };
}

/// Unwraps an option or returns from the function if none
#[macro_export]
macro_rules! unwrap_or_return {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => return,
        }
    };
}
