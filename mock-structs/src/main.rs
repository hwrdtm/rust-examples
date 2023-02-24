mod my_struct;
mod thing_that_uses_my_struct;

#[cfg(test)]
use my_struct::MockMyStruct as MyStruct;
#[cfg(not(test))]
use my_struct::MyStruct;

use thing_that_uses_my_struct::ThingThatUsesMyStruct;

fn main() {
    let my_struct = MyStruct::new();
    let thing_that_uses_my_struct = ThingThatUsesMyStruct::new(my_struct.clone());

    println!(
        "This is thing that uses my struct: {:?} with inner thing: {:?}",
        thing_that_uses_my_struct,
        thing_that_uses_my_struct.reveal_all()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thing_that_uses_my_struct() {
        // Create a matcher for calling MockMyStruct::new().
        // https://github.com/asomers/mockall/issues/424
        let ctx = MyStruct::new_context();
        ctx.expect().returning(MyStruct::default);

        // prepare
        let mut mock_my_struct = MyStruct::new();

        // describe
        mock_my_struct
            .expect_reveal()
            .returning(|| "this is definitely from a mocked struct".to_string());

        // execute
        let thing_that_uses_my_struct = ThingThatUsesMyStruct::new(mock_my_struct);

        // assert
        assert_eq!(
            thing_that_uses_my_struct.reveal_all(),
            "this is definitely from a mocked struct"
        );
    }
}
