// RUST_LOG=debug cargo run

use log::info;
use long_running_task_checker::instrument_long_running_task;

#[derive(Clone, Debug)]
struct SomeStruct {
    pub ret_val: usize,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    // Init SomeStruct
    let some_struct = SomeStruct { ret_val: 42 };

    let result = instrument_long_running_task(
        [1, 5, 5, 7, 9, 30].iter().copied().collect(),
        "long_running_task",
        long_running_task(some_struct.clone()),
    )
    .await;
    println!("Result: {:?}", result);

    // Run another long running task to show that the previous checker does not check against the
    // 30s threshold.
    long_running_task(some_struct).await?;

    Ok(())
}

async fn long_running_task(some_struct: SomeStruct) -> Result<usize, anyhow::Error> {
    info!("Starting long running task");

    // Sleep for 10 seconds
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    Ok(some_struct.ret_val)
}
