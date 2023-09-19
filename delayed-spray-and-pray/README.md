# What

Behavior:

- If the main future finishes first, then return that result without firing off the other future.
- If the main future does not finish before the delay, fire off the other future and return the result of the future that finishes first.
- Handle aborting the either future when the other one finishes.