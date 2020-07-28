mod system;

#[tokio::main]
async fn main() {
    println!("trying to connect");
    system::connect(&"192.168.122.129:9800".parse().unwrap()).await.unwrap();
    loop {
        println!("{}", system::get_key_state(0x1E).await);

    }
}