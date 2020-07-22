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

#[macro_export]
macro_rules! memory_setter {
    ($func_name: ident, $param_type:ty, $offset:expr, $base_address:expr) => {
        pub fn $func_name(&self, value: $param_type) {
            memlib::memory::write_memory($base_address + $offset, value)
        }
    };
}
