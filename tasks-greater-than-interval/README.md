# What

A little script to play around with various `MissedTickBehavior`. The default is `Burst`. Notice how `Delay` allows for the interval time to start from the last time `.tick()` was called.

```bash
tick: "2024-01-05T01:27:50.180910+00:00" count 0
sleeping for 10 seconds
tick: "2024-01-05T01:28:00.183513+00:00" count 1
tick: "2024-01-05T01:28:03.190126+00:00" count 2
tick: "2024-01-05T01:28:06.191360+00:00" count 3
sleeping for 10 seconds
tick: "2024-01-05T01:28:16.192911+00:00" count 4
tick: "2024-01-05T01:28:19.195274+00:00" count 5
tick: "2024-01-05T01:28:22.197950+00:00" count 6
sleeping for 10 seconds
tick: "2024-01-05T01:28:32.200025+00:00" count 7
tick: "2024-01-05T01:28:35.202688+00:00" count 8
tick: "2024-01-05T01:28:38.202044+00:00" count 9
sleeping for 10 seconds
tick: "2024-01-05T01:28:48.205038+00:00" count 10
tick: "2024-01-05T01:28:51.206407+00:00" count 11
tick: "2024-01-05T01:28:54.208249+00:00" count 12
sleeping for 10 seconds
tick: "2024-01-05T01:29:04.214565+00:00" count 13
...
```