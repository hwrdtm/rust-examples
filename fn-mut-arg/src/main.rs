use std::{fmt::Debug, future::Future};

#[tokio::main]
async fn main() {
    // Demonstrate the simple mapping functionality.
    let mut store = SimpleStore {
        items: vec!["a".to_string(), "b".to_string()],
        mapper_fn: Box::new(|(idx, s)| {
            println!("({}): {}", idx, s);
        }),
    };
    store.execute();

    // Demonstrate the async mapping functionality.
    let mut async_store = AsyncStore {
        items: vec!["c".to_string(), "d".to_string(), "e".to_string()],
        mapper_fn: |tup: (usize, String)| async move {
            println!("({}): {}", tup.0, tup.1);
            if tup.0 >= 1 {
                some_sleep_func(&tup.1).await;
            }
            Ok::<i32, anyhow::Error>(1)
        },
    };
    async_store.execute().await;
}

pub struct SimpleStore {
    pub items: Vec<String>,
    pub mapper_fn: Box<dyn FnMut((usize, &str))>,
}

impl SimpleStore {
    pub fn execute(&mut self) {
        for (i, item) in self.items.iter().enumerate() {
            (self.mapper_fn)((i, item));
        }
    }
}

pub struct AsyncStore<M>
where
    M: Mapper,
{
    pub items: Vec<String>,
    pub mapper_fn: M,
}

impl<M> AsyncStore<M>
where
    M: Mapper,
    <M as Mapper>::Error: Debug,
{
    pub async fn execute(&mut self) {
        for (i, item) in self.items.iter().enumerate() {
            if let Err(e) = self.mapper_fn.run((i, item.to_owned())).await {
                println!("Error: {:?}", e);
            }
        }
    }
}

pub trait Mapper {
    type Future: Future<Output = Result<Self::Item, Self::Error>>;
    type Item;
    type Error;

    fn run(&mut self, tup: (usize, String)) -> Self::Future;
}

impl<R, E, T: Future<Output = Result<R, E>>, F: FnMut((usize, String)) -> T> Mapper for F {
    type Item = R;
    type Error = E;
    type Future = T;

    fn run(&mut self, tup: (usize, String)) -> Self::Future {
        self(tup)
    }
}

async fn some_sleep_func(s: &str) {
    std::thread::sleep(std::time::Duration::from_secs(1));
    println!("some_sleep_func: {}", s);
}
