Filigram-rs
===========

Watermark image files in a folder, recursively.

<img alt="original" src="./data/original.jpg" width="300" height="300"/> <img alt="watermarked" src="./data/watermarked.jpg" width="300" height="300"/> 

Input folder is copied entirely, applying watermark on image files, depending on some exclusion/inclusion rules.
If a file is excluded from watermarking, it is simply copied to destination.

Watermarking process:
- watermark is customizable
- image is resized to a fixed size of 500x500
- process is multithreaded using `rayon` crate.

This app is meant to run on Linux and Ubuntu.

## Build and test

```console
cargo build --release && ./target/release/filigram-rs
```

For Windows:

```console
cargo build --release --target x86_64-pc-windows-gnu
```

Note: package `mingw-w64` may be required.

On Ubuntu, run `sudo apt-get install mingw-w64`

## Debug logging

```console
cargo build --release && RUST_LOG=debug ./target/release/filigram-rs
```
