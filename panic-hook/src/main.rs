use std::panic;

fn main() {
    set_panic_hook();

    // Spawn a thread that will panic
    std::thread::spawn(|| {
        panic!("This is a panic from a thread!");
    });

    // Sleep for 2 seconds to allow the thread to panic
    std::thread::sleep(std::time::Duration::from_secs(2));
    panic!("This is a panic!");
}

pub fn set_panic_hook() {
    panic::set_hook(Box::new(move |e| {
        println!("HI {:?}", e);
    }));
}
