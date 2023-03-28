# tower-serve-dir-perf

This repo demonstrates performance quirks of tower's [ServeDir](https://docs.rs/tower-http/latest/tower_http/services/struct.ServeDir.html) middleware, and attempts to investigate the root cause.

It has currently isolated high latency to any time the `Body` returns `Poll::Pending` on a call to `poll_data`. Calling the waker immediately before returning doesn't help.

To see results for yourself, run the server:

```shell
$ cargo run -r
```

Then run the benchmarking script (requires `wrk` to be installed).

```shell
$ ./bench.sh
```

The different paths function as follows:

- /single: Returns several lines of data as a single String.
- /body: Returns several lines of data via repeatedly calling `Body::poll_data`, once per line.
- /delayed_body: Does the same as `/body`, except that it alternates between a `Pending` response and a real line.

On my machine, all results return in a couple hundred microseconds, except when delayed_body returns a `Pending` response. When this happens, latency immediately jumps to about 50ms.

## Experiments

- Tried setting the capacity of the `ReaderStream` to the length of the file, instead of to 64kB. No change.
- Tried using `AsyncReadExt::read_to_end` with a vec. This had similar performance to `read` (< 2x).
- Tried using `AsyncReadExt::read` to repeatedly read into a vec buffer. This had similar performance to above (~ 2x `read`).
- Tried using `ReaderStream` with `StreamExt::fold` to read data into a `BytesMut` buffer, and return that once. This performed similar to above(~ 2x `read`).
- Tried removing all file I/O, and simply returning string data via a Body. This is the current state, and shows the large latency spike whenever `Poll::Pending` is returned.
