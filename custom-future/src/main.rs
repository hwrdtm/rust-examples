use std::time::Duration;

use custom::TimerFuture;

mod custom {
    use std::{
        future::Future,
        sync::{Arc, Mutex},
        task::{Poll, Waker},
        thread,
        time::Duration,
    };

    pub struct TimerFuture {
        shared_state: Arc<Mutex<SharedState>>,
    }

    struct SharedState {
        completed: bool,
        waker: Option<Waker>,
    }

    impl TimerFuture {
        pub fn new() -> Self {
            let shared_state = Arc::new(Mutex::new(SharedState {
                completed: false,
                waker: None,
            }));

            TimerFuture { shared_state }
        }
    }

    impl Future for TimerFuture {
        type Output = ();

        fn poll(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            if self.shared_state.lock().unwrap().completed {
                return Poll::Ready(());
            }

            // Set the waker
            let waker = cx.waker().clone();
            let mut shared_state = self.shared_state.lock().unwrap();
            shared_state.waker = Some(waker);

            // Spawn a thread that tells the executor to wake this task after 2s.
            let thread_shared_state = self.shared_state.clone();
            std::thread::spawn(move || {
                thread::sleep(Duration::from_secs(2));

                let mut shared_state = thread_shared_state.lock().unwrap();
                shared_state.completed = true;
                if let Some(waker) = shared_state.waker.take() {
                    waker.wake();
                }
            });

            Poll::Pending
        }
    }
}

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
