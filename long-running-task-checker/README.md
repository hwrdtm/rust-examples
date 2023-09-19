# What

Here is the output of running `RUST_LOG=debug cargo run`:

```log
[2023-09-19T00:06:11Z INFO  long_running_task_checker] Starting long running task
[2023-09-19T00:06:12Z WARN  long_running_task_checker] [long_running_task] Elapsed: 1.002882458s
[2023-09-19T00:06:17Z WARN  long_running_task_checker] [long_running_task] Elapsed: 5.010781542s
[2023-09-19T00:06:19Z WARN  long_running_task_checker] [long_running_task] Elapsed: 7.015596083s
[2023-09-19T00:06:21Z WARN  long_running_task_checker] [long_running_task] Elapsed: 9.020249s
[2023-09-19T00:06:21Z INFO  long_running_task_checker] [long_running_task] Elapsed (Total): 10.002476s
Result: Ok(42)
[2023-09-19T00:06:21Z INFO  long_running_task_checker] Starting long running task
```