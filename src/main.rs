use std::time::Instant;

mod system;

fn main() {
    system::connect(&"192.168.122.129:9800".parse().unwrap()).unwrap();
    loop {
        let start = Instant::now();
        println!("{}", system::get_key_state(0x01));
        let time = Instant::now() - start;
        println!("{}", time.as_micros() as f32 / 1000.0);
    }
}
