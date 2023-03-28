# tower-serve-dir-perf

This repo demonstrates performance quirks of tower's [ServeDir](https://docs.rs/tower-http/latest/tower_http/services/struct.ServeDir.html) middleware, and attempts to investigate the root cause.

To see results for yourself, run the server:

```shell
$ cargo run -r
```

Then run the benchmarking script (requires `wrk` to be installed).

```shell
$ ./bench.sh
```

This returns static files in three ways:

- /serve_dir: Returns files using tower's `ServeDir` middleware.
- /read: Returns contents by reading it all at once using `tokio::fs::read`.
- /stream: Returns file contents in a streaming response, replicating the core behavior of `ServeDir`.

On my machine, `/read` returns in about 0.5ms on average, whereas the others take about 40-50ms.

## Experiments

- Tried setting the capacity of the `ReaderStream` to the length of the file, instead of to 64kB. No change.
- Tried using `AsyncReadExt::read_to_end` with a vec. This had similar performance to `read` (< 2x).
- Tried using `AsyncReadExt::read` to repeatedly read into a vec buffer. This had similar performance to above (~ 2x `read`).
- Tried using `ReaderStream` with `StreamExt::fold` to read data into a `BytesMut` buffer, and return that once. This performed similar to above(~ 2x `read`).
