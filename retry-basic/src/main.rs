use std::sync::atomic::{AtomicUsize, Ordering};

#[tokio::main]
async fn main() {
    todo!("Implement the retry utility itself");
}

/// Fallible async function that fails the first 2 times it is called, then succeeds.
async fn fallible_async_function_with_param(param: usize) -> Result<usize, ()> {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
    if counter < 3 {
        // Log with counter
        println!("Failing with counter {}", counter);
        Err(())
    } else {
        println!("Succeeding");
        Ok(param)
    }
}
