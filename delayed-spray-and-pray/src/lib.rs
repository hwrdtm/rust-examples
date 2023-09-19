use log::debug;
use tokio::select;

/// Depending on whichever future finishes first, the other future gets dropped and the executor (should)
/// no longer be polling it.
pub async fn delayed_spray_and_pray<F, S>(
    delay_ms: u64,
    main_f_name: S,
    main_f: F,
    other_f_name: S,
    other_f: F,
) -> F::Output
where
    F: core::future::Future,
    S: AsRef<str> + Send + Clone + 'static,
{
    select! {
        main_res = async {
            // Execute the main futurue.
            let main_res = main_f.await;
            debug!("[{}] finished", main_f_name.as_ref());

            main_res
        } => main_res,
        other_res = async {
            // Sleep for the delay.
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            debug!("[{}] took longer than {}ms, firing off [{}]", main_f_name.as_ref(), delay_ms, other_f_name.as_ref());

            // Execute the other future.
            let other_res = other_f.await;
            debug!("[{}] finished", other_f_name.as_ref());

            other_res
        } => other_res,
    }
}
