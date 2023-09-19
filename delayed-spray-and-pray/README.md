# What

Behavior:

- If the main future finishes first, then return that result without firing off the other future.
- If the main future does not finish before the delay, fire off the other future and return the result of the future that finishes first.
- Handle aborting the either future when the other one finishes.

# Example Logs

```log
[2023-09-19T20:06:04Z INFO  delayed_spray_and_pray] Starting long running task
[2023-09-19T20:06:04Z DEBUG delayed_spray_and_pray] [main_task] took longer than 500ms, firing off [backup_task]
[2023-09-19T20:06:04Z INFO  delayed_spray_and_pray] Starting long running task
[2023-09-19T20:06:08Z DEBUG delayed_spray_and_pray] [backup_task] finished
[2023-09-19T20:06:08Z INFO  delayed_spray_and_pray] Starting long running task
```