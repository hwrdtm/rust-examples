use std::sync::Arc;

fn main() {
    println!("\nExample 1: RwLock write lock blocks read lock\n");
    write_lock_blocks();

    println!("\nExample 2: RwLock deadlock\n");
    always_deadlocks();
}

/// Example to demonstrate a deadlock.
/// 
/// The reason for this is that the write lock is unable to acquire
/// a write lock because the read lock is held by the main thread,
/// and when the main thread is acquiring another read lock, it
/// blocks too, so the original read lock is never released (as it
/// never goes out of scope) and the main thread is in a deadlock.
fn always_deadlocks() {
    // Create a new RwLock.
    let lock = Arc::new(std::sync::RwLock::new(9999));

    // Read the lock.
    let mut _rg = lock.read().unwrap();

    // Spawn a new thread to write the lock.
    let lock_2 = lock.clone();
    let write_thread = std::thread::spawn(move || {
        println!("write_thread spawned");

        // Note the time.
        let start = std::time::Instant::now();

        // Acquire a write lock.
        let mut write_res = lock_2.write();
        if write_res.is_err() {
            println!("write_res is err");
        }

        let mut guard = write_res.unwrap();

        // Print the value.
        println!("write: {}", *guard);

        // Update the value.
        *guard += 1;

        // Note the time.
        let end = std::time::Instant::now();

        // Print the elapsed time.
        println!("[write_thread] elapsed: {:?}\n", end - start);
    });

    // Sleep for 100ms to simulate a long read.
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Read the lock again.
    let mut _rg = lock.read().unwrap();

    print!("final: {}", *lock.read().unwrap());
}

/// Example to demonstrate write lock being blocked by read lock.
/// 
/// The reason for this is that the call to obtain a write lock
/// blocks until all write and read locks are released.
fn write_lock_blocks() {
    // Create a new RwLock.
    let lock = Arc::new(std::sync::RwLock::new(9999));

    // Spawns a new thread that reads from the lock.
    let lock_1 = lock.clone();
    let read_thread = std::thread::spawn(move || {
        // Note the time.
        let start = std::time::Instant::now();

        println!("read_thread spawned");
        let mut read_res = lock_1.read();
        if read_res.is_err() {
            println!("read_res is err");
        }

        let guard = read_res.unwrap();

        // Print the value.
        println!("read value: {}", *guard);

        // Sleep for 100ms to simulate a long read.
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Note the time.
        let end = std::time::Instant::now();

        // Print the elapsed time.
        println!("[read_thread] elapsed: {:?}\n", end - start);
    });

    // Spawn a new thread to write the lock.
    let lock_2 = lock.clone();
    let write_thread = std::thread::spawn(move || {
        // Wait for the read thread to start.
        std::thread::sleep(std::time::Duration::from_millis(10));

        println!("write_thread spawned");

        // Note the time.
        let start = std::time::Instant::now();

        // Acquire a write lock.
        let mut write_res = lock_2.write();
        if write_res.is_err() {
            println!("write_res is err");
        }

        let mut guard = write_res.unwrap();

        // Print the value.
        println!("write: {}", *guard);

        // Update the value.
        *guard += 1;

        // Note the time.
        let end = std::time::Instant::now();

        // Print the elapsed time.
        println!("[write_thread] elapsed: {:?}\n", end - start);
    });

    // Wait for the threads to finish.
    read_thread.join().unwrap();
    write_thread.join().unwrap();

    // Print the value.
    println!("final: {}", *lock.read().unwrap());
}
