use std::sync::atomic::{AtomicUsize, Ordering};
use tokio_retry::strategy::{jitter, FixedInterval};
use tokio_retry::{Action, Retry};

#[tokio::main]
async fn main() {
    // Invoke the async function
    let thing = fallible_async_function_with_param(2);
    let res = fallible_async_function_with_param(1).await;

    let res = call_with_retry(|| fallible_async_function_with_param(1)).await;

    // Print the result
    println!("Result: {:?}", res);
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

/// Wraps a function that returns a future in a retry strategy.
/// The universal retry strategy we will use for all our async functions is a fixed interval strategy
/// that will retry the function 2 times, totalling 3 attempts.
async fn call_with_retry<A: Action>(
    action: A,
) -> Result<<A as Action>::Item, <A as Action>::Error> {
    let retry_strategy = FixedInterval::new(std::time::Duration::from_secs(5))
        .take(2)
        .map(jitter);

    Retry::spawn(retry_strategy, action).await
}
