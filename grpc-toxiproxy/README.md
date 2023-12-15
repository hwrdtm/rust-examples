# Hello World GRPC Client-Server with Toxiproxy Fault Injection

The setup was done following the tutorial at https://github.com/hyperium/tonic/blob/d5c14fa20623a189771d993cb019697565393df6/examples/helloworld-tutorial.md

## Manual Testing Toxiproxy

- First start [Toxiproxy](https://github.com/Shopify/toxiproxy)
- Spin up server with `cargo run --bin server`
- Inject the faults with `cargo run --bin faults`
- Spin up the client with `cargo run --bin client`
- Observe the effects of an injected latency fault, eg. the log `Request took 5011ms`