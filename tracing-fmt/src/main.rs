use tracing::{debug, error, info, instrument};
use tracing_fmt::init_subscriber;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    let sub = init_subscriber().expect("Failed to create subscriber");
    sub.init();

    let massive_struct =
        MassiveStruct::new("This is a super long string repeated multiple times".to_string());
    massive_struct.first_layer().await;
}

#[derive(Debug)]
struct MassiveStruct {
    long_text: String,
}

impl MassiveStruct {
    fn new(string: String) -> Self {
        Self {
            long_text: string.repeat(10),
        }
    }

    #[instrument]
    async fn first_layer(&self) {
        info!("HI!!!");

        // Call second_layer 5 times.
        for i in 0..10 {
            self.second_layer().await;

            if i % 2 == 0 {
                error!("An error occurred on iteration {}", i);
            }
        }
    }

    #[instrument]
    async fn second_layer(&self) {
        debug!("OMG!");

        error!("Something went wrong!");
    }
}
