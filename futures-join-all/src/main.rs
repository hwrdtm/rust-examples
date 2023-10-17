// RUST_LOG=debug cargo run

use log::{debug, info};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    // Mode 1: Using join_all to drive parallel futures at the same time.
    let mut fut = vec![];
    for i in 0..10 {
        fut.push(timeoutsuccess(format!("thread_{}", i)));
    }

    // Sleep for 5s - you will not see logs during these 5 seconds.
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let finished_futs = futures::future::join_all(fut).await;
    info!("Finished futures: {:?}", finished_futs);

    // Mode 2: Sequentially awaiting to drive futures one at a time.
    let mut fut = vec![];
    for i in 0..10 {
        fut.push(timeoutsuccess(format!("thread_{}", i)));
    }
    for f in fut {
        let finished_fut = f.await?;
        info!("Finished future: {:?}", finished_fut);
    }

    Ok(())
}

// Prints a log every second for 10s and then returns.
async fn timeoutsuccess(thread_name: String) -> Result<String, anyhow::Error> {
    let mut i = 0;
    while i < 10 {
        debug!(
            "[timeoutsuccess::{}] Sleeping for 1s, i: {}",
            thread_name, i
        );
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        i += 1;
    }

    Ok(thread_name)
}
