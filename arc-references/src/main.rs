use std::sync::Arc;

#[derive(Clone)]
pub struct Context {
    pub arc_ref: Arc<Config>,
}

pub struct Config {
    pub key: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            key: "value".to_string(),
        }
    }
}

fn main() {
    // This script demonstrates how the number of reference count changes when Context,
    // which contains an Arc reference to Config, is deep-cloned.
    let cfg = Arc::new(Config::new());
    let ctx = Context {
        arc_ref: Arc::clone(&cfg),
    };

    println!("Reference count of cfg: {}", Arc::strong_count(&cfg));

    let ctx2 = ctx.clone();

    println!("Reference count of cfg: {}", Arc::strong_count(&cfg));
}
