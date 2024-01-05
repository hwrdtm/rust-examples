use std::time::Duration;

use tokio::time::interval;

#[tokio::main]
async fn main() {
    let mut interval = interval(Duration::from_secs(3));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut count = 0;

    loop {
        interval.tick().await;

        // print the current timestamp as ISO 8601
        let now = chrono::Utc::now();
        println!("tick: {:?} count {:?}", now.to_rfc3339(), count);

        // When counter is divisible by 3, sleep for 10 seconds
        if count % 3 == 0 {
            println!("sleeping for 10 seconds");
            tokio::time::sleep(Duration::from_secs(10)).await;
        }

        // Increment
        count += 1;
    }
}
