# Benchmark different ways of reading from files while trying to avoid memset

The current stable Rust release has no safe way to read from files into a
buffer, without having to initialize that buffer first. These benchmarks
show that this initialization can add quite some overhead.

Stabilization of such a feature is being worked on in
[RFC 2930](https://github.com/rust-lang/rust/issues/78485).

An alternative for reading into a `Vec<u8>` could be to use
[`Read::take`](https://doc.rust-lang.org/std/io/trait.Read.html#method.take).

```
file.take(size).read_to_end(vec)`
```

Benchmark results from i9-11900KB: 

```
readchunks/read_exact_alloc_zeroed
                        time:   [57.987 µs 58.085 µs 58.208 µs]
                        thrpt:  [16.777 GiB/s 16.813 GiB/s 16.841 GiB/s]
readchunks/read_exact_alloc_uninit
                        time:   [40.572 µs 41.056 µs 41.549 µs]
                        thrpt:  [23.504 GiB/s 23.786 GiB/s 24.070 GiB/s]
readchunks/read_exact_zeroed
                        time:   [57.294 µs 57.866 µs 58.496 µs]
                        thrpt:  [16.694 GiB/s 16.876 GiB/s 17.045 GiB/s]
readchunks/read_exact_uninit
                        time:   [40.093 µs 40.317 µs 40.589 µs]
                        thrpt:  [24.060 GiB/s 24.222 GiB/s 24.357 GiB/s]
readchunks/read_take_to_end
                        time:   [41.473 µs 41.562 µs 41.670 µs]
                        thrpt:  [23.435 GiB/s 23.496 GiB/s 23.547 GiB/s]
readchunks/read_buf     time:   [39.805 µs 39.925 µs 40.071 µs]
                        thrpt:  [24.371 GiB/s 24.460 GiB/s 24.534 GiB/s]
```