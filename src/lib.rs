//!Static friendly data structs
//!
//!## Available data structures
//!
//!- [Array](struct.Array.html)
//!- [RingBuffer](ring/struct.RingBuffer.html)
//!
//!## Crate features
//!
//!- `std` - Enables some std interfaces (e.g. `Write`) implementations.
//!- `serde` - Enables serialization/deserialization implementations.

#![no_std]
#![deny(warnings)]
#![warn(missing_docs)]
#![allow(clippy::style)]

#[cfg(feature = "std")]
extern crate std;

mod array;
pub use array::Array;
pub mod ring;
pub use ring::RingBuffer;

#[cfg(feature = "serde")]
mod serde;
