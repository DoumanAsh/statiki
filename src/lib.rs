//!Static friendly data structs
//!
//!## Available data structures
//!
//!- [RingBuffer](ring_buffer/index.html)
//!- [Vec](vec/index.html)

#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

pub mod ring_buffer;
pub mod vec;
