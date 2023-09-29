use tokio_retry::strategy::{jitter, FixedInterval};
use tokio_retry::{Action, Retry, RetryIf, Condition};

/// Wraps a function that returns a future in a retry strategy.
/// The universal retry strategy we will use for all our async functions is a fixed interval strategy
/// that will retry the function 2 times, totalling 3 attempts.
pub async fn call_with_retry<A: Action>(
    action: A,
) -> Result<<A as Action>::Item, <A as Action>::Error> {
    let retry_strategy = FixedInterval::new(std::time::Duration::from_secs(5))
        .take(2)
        .map(jitter);

    Retry::spawn(retry_strategy, action).await
}

/// Wraps a function that returns a future in a retry strategy depending on a condition.
/// 
/// The universal retry strategy we will use for all our async functions is a fixed interval strategy
/// that will retry the function 2 times, totalling 3 attempts.
pub async fn call_with_retry_condition<A: Action, C: Condition<A::Error>>(
    action: A,
    condition: C,
) -> Result<<A as Action>::Item, <A as Action>::Error> {
    let retry_strategy = FixedInterval::new(std::time::Duration::from_secs(5))
        .take(2)
        .map(jitter);

    RetryIf::spawn(retry_strategy, action, condition).await
}
