#[cfg(test)]
use crate::my_struct::MockMyStruct as MyStruct;
#[cfg(not(test))]
use crate::my_struct::MyStruct;

#[derive(Debug, Clone)]
pub struct ThingThatUsesMyStruct {
    pub my_struct: MyStruct,
}

impl ThingThatUsesMyStruct {
    pub fn new(my_struct: MyStruct) -> Self {
        Self { my_struct }
    }

    pub fn reveal_all(&self) -> String {
        self.my_struct.reveal()
    }
}
