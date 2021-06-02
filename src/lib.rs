//!Static friendly data structs
//!
//!## Available data structures
//!
//!- [Array](vec/index.html)
//!
//!## Crate features
//!
//!- `std` Enables some std interfaces (e.g. `Write`) implementations.

#![no_std]
//#![deny(warnings)]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

#[cfg(feature = "std")]
extern crate std;

mod vec;
pub use vec::Array;
