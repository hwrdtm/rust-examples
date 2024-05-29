use std::collections::BTreeSet;

use log::{info, trace, warn};

pub async fn instrument_long_running_task<F, S>(
    sec_thresholds: BTreeSet<u64>,
    f_name: S,
    f: F,
) -> F::Output
where
    F: core::future::Future,
    S: AsRef<str> + Send + Clone + 'static,
{
    let start = std::time::Instant::now();

    // Kick off the checker.
    let f_name_clone = f_name.clone();
    let handle = tokio::spawn(async move {
        log_at_thresholds(sec_thresholds, f_name_clone).await;
    });

    // Execute the future.
    let res = f.await;

    // Calculate the total elapsed time.
    let elapsed = start.elapsed();
    info!(
        "[{}] Elapsed (Total): {:?}s",
        f_name.as_ref(),
        elapsed.as_secs_f64()
    );

    // Cancel the checker now that the future has finished.
    handle.abort();

    // Return the value from the future.
    res
}

async fn log_at_thresholds<S>(sec_thresholds: BTreeSet<u64>, f_name: S)
where
    S: AsRef<str>,
{
    trace!("sec_thresholds: {:?}", sec_thresholds);

    let start = std::time::Instant::now();
    let mut sec_thresholds = sec_thresholds;

    while let Some(next_threshold_secs) = sec_thresholds.pop_first() {
        trace!("Threshold: {:?}", next_threshold_secs);

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let elapsed = start.elapsed();

        if elapsed.as_secs() >= next_threshold_secs {
            warn!(
                "[{}] Elapsed: {:?}s",
                f_name.as_ref(),
                elapsed.as_secs_f64()
            );
        } else {
            // push the element back in the set
            sec_thresholds.insert(next_threshold_secs);
        }
    }
}
