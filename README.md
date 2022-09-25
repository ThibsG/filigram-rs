Filigram-rs
===========

This library is a convenient wrapper to watermark image files in a folder, recursively.

<p align="center">
    <img alt="original" src="./data/original.jpg" width="300" height="300"/> 
    <img alt="watermarked" src="./data/watermarked.jpg" width="300" height="300"/>
</p>

Input folder is copied entirely, applying watermark on image files, depending on some exclusion/inclusion rules.
If a file is excluded from watermarking, it is simply copied to destination without any change.

Watermarking process:
- watermark text is customizable
- image is resized to a fixed size of 500x500
- process is multithreaded using `rayon` crate.

## Compatibility

This library is compatible with:
- Linux (x86_64-unknown-linux-gnu)
- Windows (x86_64-pc-windows-gnu)
- WASI (wasm32-wasi)

## Build the library

```console
cargo build --release
```

For Windows:

```console
cargo build --release --target x86_64-pc-windows-gnu
```

Note: package `mingw-w64` may be required for cross-compilation.

On Ubuntu, run `sudo apt-get install mingw-w64`

For WASI:

```console
cargo build --release --target wasm32-wasi
```

## Run the example

A simple example is provided in the subfolder `examples` to illustrate how to use the library.

```console
cargo run --release --example filigram
```
