# vox-rs

A Rust MagicaVoxel .vox file loader and writer.

Ported from [opengametools](https://github.com/jpaver/opengametools/)

Some usage examples are available in [examples](./examples), including a simple CLI raytracer.

All public APIs are documented here: https://docs.rs/vox-rs

## no_std support

To compile without std, do `vox-rs = { version = "...", default-features = false, features = ["no_std"] }`

## Comparison with [dot_vox](https://github.com/dust-engine/dot_vox)

* vox-rs has no dependencies (doesn't depend on nom)
* vox-rs automatically resolves animation looping and interpolation
* vox-rs can bake scenegraph hierarchies and keyframes down to flat instances, involves no walking down trees
* vox-rs can parse camera, lighting and metadata chunks
* vox-rs allows for merging multiple .vox files and automatic model deduplication
* vox-rs provides the user with pre-transformed matrices
* vox-rs supports `#![no_std]`
