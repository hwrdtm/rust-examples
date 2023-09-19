// RUST_LOG=debug cargo run

use delayed_spray_and_pray::delayed_spray_and_pray;
use log::info;

#[derive(Clone, Debug)]
struct SomeStruct {
    pub ret_val: u64,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    // Test 1: other future finishes first.
    let res = delayed_spray_and_pray(
        500,
        "main_task",
        long_running_task(SomeStruct { ret_val: 10 }),
        "backup_task",
        long_running_task(SomeStruct { ret_val: 4 }),
    )
    .await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 4);

    // Test 2: main future finishes first.
    let res = delayed_spray_and_pray(
        5000,
        "main_task",
        long_running_task(SomeStruct { ret_val: 4 }),
        "backup_task",
        long_running_task(SomeStruct { ret_val: 10 }),
    )
    .await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 4);

    let _ = long_running_task(SomeStruct { ret_val: 10 }).await;

    Ok(())
}

async fn long_running_task(some_struct: SomeStruct) -> Result<u64, anyhow::Error> {
    info!("Starting long running task");

    // Sleep for 10 seconds
    tokio::time::sleep(std::time::Duration::from_secs(some_struct.ret_val)).await;

    Ok(some_struct.ret_val)
}
