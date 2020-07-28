mod system;

fn main() {
    system::connect(&"192.168.122.129:9800".parse().unwrap()).unwrap();
    loop {
        println!("{}", system::get_key_state(0x01));
    }
}