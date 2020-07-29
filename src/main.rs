use std::time::Instant;

mod system;

fn main() {
    println!("Connecting...");
    system::connect(&"192.168.122.129:9800".parse().unwrap()).unwrap();
    println!("Connected...");
    system::move_mouse_relative(5, 0);
    // dbg!(system::get_key_state(5));
}
