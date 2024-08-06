fn main() {
    let rt_1 = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    let rt_2 = tokio::runtime::Runtime::new().expect("Failed to create runtime");

    rt_1.spawn(async {
        task_that_does_normal_work("rt_1").await;
    });

    rt_1.spawn(async {
        task_that_sleeps_thread("rt_1").await;
    });

    // Dummy code to block forever.
    rt_2.block_on(async {
        // Print a log line each second
        loop {
            let now = std::time::SystemTime::now();
            println!("[main_thread] Doing some work at {:?}", now);
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    })
}

async fn task_that_does_normal_work(rt_name: &str) {
    // Print a log line each second
    loop {
        let now = std::time::SystemTime::now();
        println!("[{:?}] Doing some work at {:?}", rt_name, now);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

async fn task_that_sleeps_thread(rt_name: &str) {
    // Sleep the thread after 5 seconds
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let now = std::time::SystemTime::now();
    println!("[{:?}] Sleeping the std::thread at {:?}", rt_name, now);

    std::thread::sleep(std::time::Duration::from_secs(1000));
}