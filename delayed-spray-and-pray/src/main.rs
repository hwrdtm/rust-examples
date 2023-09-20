// RUST_LOG=debug cargo run

use delayed_spray_and_pray::delayed_spray_and_pray;
use log::info;

#[derive(Clone, Debug, PartialEq)]
struct SomeStruct {
    pub ret_val: u64,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let client = reqwest::Client::builder()
        .build()
        .unwrap();

    // Test 1: other future finishes first.
    let res = delayed_spray_and_pray(
        500,
        "main_task",
        client.get("https://www.google.com/").send(),
        "backup_task",
        async {
            client.get("https://google.com/").send()
        }.await,
    )
    .await;
    assert!(res.is_ok());
    println!("{:?}", res.unwrap());

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
    assert_eq!(res.unwrap(), SomeStruct { ret_val: 4 });

    let _ = long_running_task(SomeStruct { ret_val: 10 }).await;

    Ok(())
}

async fn long_running_task(some_struct: SomeStruct) -> Result<SomeStruct, anyhow::Error> {
    info!("Starting long running task");

    // Sleep for 10 seconds
    tokio::time::sleep(std::time::Duration::from_secs(some_struct.ret_val)).await;

    Ok(some_struct.clone())
}
