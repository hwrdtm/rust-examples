# Rocket Concurrency

Demo to show concurrent requests against a Rocket server.

# Usage

- Install [k6](https://grafana.com/docs/k6/latest/get-started/running-k6/)
- Run `k6 run script_tokio.js` to show the number of concurrent requests handled when we are making Tokio threads (green threads) to sleep
- Run `k6 run script_core.js` to show the number of concurrent requests handled when we are making the CPU core threads to sleep


## Analysis

Say we are running on a machine with 8 cores. By default, Rocket assigns the number of worker (Tokio) threads to be the `number of logical cores x 2`, so Rocket would assign 16 Tokio threads to the server.

- Tokio sleep: Whenever `tokio::time::sleep` is called and `await`ed, the code yields to the Tokio runtime, so that the worker is released to handle a new request. This is why in this case, we can handle all 30 VUs (concurrent requests) from k6.
- Core sleep: Whenever `std::thread::sleep` is called, we are actually sleeping on the CPU core and keeping it busy (nothing is yielded), and so the server cannot handle more than 8 VUs (concurrent requests) from k6.