#[derive(Debug, Clone)]
pub struct MyStruct {
    thing: String,
}

impl MyStruct {
    pub fn new() -> Self {
        Self {
            thing: "surprise!".to_string(),
        }
    }

    pub fn reveal(&self) -> String {
        self.thing.clone()
    }
}

mockall::mock! {
    #[derive(Debug)]
    pub MyStruct {
        pub fn new() -> Self;
        pub fn reveal(&self) -> String;
    }
    impl Clone for MyStruct {
        fn clone(&self) -> Self;
    }
}
