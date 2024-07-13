use std::time::Duration;

use custom_future::TimerFuture;

#[tokio::main]
async fn main() {
    let f = timer_fut();

    tokio::time::sleep(Duration::from_secs(3)).await;

    let now = std::time::Instant::now();
    let _ = f.await;
    println!("Time since polling: {:?}", now.elapsed());
}

fn timer_fut() -> TimerFuture {
    TimerFuture::new()
}
