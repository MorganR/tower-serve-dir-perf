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

The results can be found in `./results.txt`.
