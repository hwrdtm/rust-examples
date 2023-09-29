# Distributed Tracing Example

This is an example demonstrating distributed tracing across network boundaries using [`tokio-rs/tracing`](https://github.com/tokio-rs/tracing) that is compliant with OpenTelemetry via [`tracing-opentelemetry`](https://github.com/tokio-rs/tracing-opentelemetry).

## Getting Started

1. Start your local Jaeger agent

```bash
docker run -d -p6831:6831/udp -p6832:6832/udp -p16686:16686 -p14268:14268 jaegertracing/all-in-one:latest
```

2. Start the server

3. Run the client

4. Navigate to http://localhost:16686/ to search for the collected trace.

**NOTE**: You may have to run the client several times / wait for some time before the results show up in Jaeger. Not sure what it is, maybe there is some delay with flushing the data from the agent to the collector API.

### Client

Run with:

```bash
RUST_LOG=debug cargo run
```

### Server

Run with:

```bash
RUST_LOG=debug cargo run
```