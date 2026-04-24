#![cfg_attr(feature = "no_std", no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(all(feature = "std", feature = "no_std"))]
compile_error!(
    "`std` and `no_std` features are mutually exclusive. Use `default-features = false, features = [\"no_std\"]` for no_std builds."
);

extern crate alloc;

mod codec;
mod types;

pub use types::*;

#[cfg(test)]
mod tests;
