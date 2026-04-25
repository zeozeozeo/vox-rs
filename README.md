# vox-rs

A Rust MagicaVoxel .vox file loader and writer.

Ported from [opengametools](https://github.com/jpaver/opengametools/)

Some usage examples are available in [examples](./examples), including a simple CLI raytracer.

All public APIs are documented here: https://docs.rs/vox-rs

## no_std support

To compile without std, do `vox-rs = { version = "...", default-features = false, features = ["no_std"] }`
