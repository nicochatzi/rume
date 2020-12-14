//!
//!
//!

#![feature(min_const_generics)]

#![deny(warnings)]
// #![deny(clippy::all, clippy::pedantic)]

#![no_std]


extern crate alloc;

#[macro_use]
pub mod graph;
pub use graph::*;

pub mod dsp;
pub use dsp::*;
