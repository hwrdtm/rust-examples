# Serde with Enums

This example shows the behavior of serde with incorrectly matching nested structures versus correct ones, using enums. **The learning is that, incorrectly matching the type definition can lead to the entire enum not to be deserialized altogether**.

With `sample_incorrect.json`, we see that the value of `property` is a `String`, which **incorrectly** matches the type definition of `Something::property`, which results in the incorrect **deserialization** of the structure, as shown in the logs when you run `cargo run`. The same incorrect result is shown when the structure is serialized back into `sample_incorrect_serialized.json`.

On the other hand, with `sample.json`, we see that the value of `property` is a floating number, which **correctly** matches the type definition of `Something::property`, which results in the correct **deserialization** of the structure, also shwon in the logs when you run `cargo run`. The same correct result is shown when the structure is serialized back into `sample_serialized.json`. 
