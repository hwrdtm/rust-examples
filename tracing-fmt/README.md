# Custom `tracing_subscriber::fmt::layer::Layer`

This example demonstrates how to use a customized version of the `tracing_subscriber::fmt::layer::Layer` layer code for achieving the following:

- Specify the event formatter to prefix each event (log line) with a string
- Specify the event formatter to omit writing span context details for each event
- Use the `CustomFieldFormatter` to only care about recording a field when it is named `message`, which should only pertain to logs.

## Instructions

- Run `cargo run` to see the prefix being used, and the span context to be omitted
- Run `cargo run` with `.with_event_scope(true)` and see the span context included along with its fields
- Run `cargo run --features ignore-fields` with `.with_event_scope(true)` and see the span context included without its fields.