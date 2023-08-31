#![deny(warnings)]
#![cfg_attr(not(feature = "std"), no_std)]

pub use rume_core::*;
pub use rume_macros::*;

#[cfg(feature = "std")]
pub mod processors;

#[cfg(feature = "std")]
pub use processors::*;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
pub use alloc::boxed::Box;

#[cfg(feature = "lab")]
pub mod lab;
