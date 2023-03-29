fn main() {
    println!("Hello, world!");

    // This is a test to see whether building with `cargo run --features hidden` will
    // print "Hello, world! (mock)", without Cargo.toml having a `features` section.
    //
    // It does not and would not even compile. Try commenting out the `features` section
    // in Cargo.toml.
    #[cfg(feature = "hidden")]
    {
        println!("Hello, world! (mock)");
    }
}
