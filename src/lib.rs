//!Static friendly data structs
//!
//!## Available data structures
//!
//!- [Array]
//!- [RingBuffer]
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

pub mod array;
pub use array::Array;
pub mod ring;
pub use ring::RingBuffer;

#[cfg(feature = "serde")]
mod serde;
