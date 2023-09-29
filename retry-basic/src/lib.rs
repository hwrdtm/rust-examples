use async_trait::async_trait;

#[async_trait]
pub trait RetryStrategy {
    async fn execute(&mut self) -> Result<(), anyhow::Error>;
}

pub struct FixedInterval {
    interval: std::time::Duration,
    attempted_executions: usize,
    max_attempts: usize,
}

impl FixedInterval {
    pub fn new(interval: std::time::Duration, max_attempts: usize) -> Self {
        FixedInterval {
            interval,
            attempted_executions: 0,
            max_attempts,
        }
    }
}

#[async_trait]
impl RetryStrategy for FixedInterval {
    async fn execute(&mut self) -> Result<(), anyhow::Error> {
        if self.attempted_executions == self.max_attempts {
            return Err(anyhow::anyhow!("Max attempts reached"));
        }

        // Sleep for the interval
        tokio::time::sleep(self.interval).await;

        // Increment the number of attempted executions
        self.attempted_executions += 1;

        Ok(())
    }
}
