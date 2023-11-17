fn main() {
    // Log a message each second with the current time.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        println!("Current time: {:?}", std::time::SystemTime::now());
    }
}
