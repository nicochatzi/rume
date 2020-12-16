//!
//!
//!

#![deny(warnings)]
#![cfg_attr(not(any(feature = "std", test)), no_std)]

#[macro_use]
pub mod graph;
pub use graph::*;

pub mod dsp;
pub use dsp::*;

#[cfg(feature = "std")]
pub mod lib {
    pub use std::{
        boxed::Box,
        rc::{Rc, Weak},
        vec,
        vec::Vec,
    };
}

#[cfg(not(feature = "std"))]
pub mod lib {
    pub use alloc::{
        boxed::Box,
        rc::{Rc, Weak},
        vec,
        vec::Vec,
    };
}

#[cfg(not(feature = "std"))]
extern crate alloc;

/// A trait used to extend
/// the functional f32 methods
/// that are provided in the
/// standard library. This allows
/// using the same `2._f32.powf(2.)`
/// syntax that `std::f32` offers
/// but in a `no_std` environment.
#[cfg(not(feature = "std"))]
pub trait F32Extension {
    fn powf(self, exp: f32) -> f32;
    fn log2(self) -> f32;
    fn log10(self) -> f32;
    fn sin(self) -> f32;
    fn exp(self) -> f32;
}

#[cfg(not(feature = "std"))]
impl F32Extension for f32 {
    #[inline(always)]
    fn powf(self, exp: f32) -> f32 {
        libm::powf(self, exp)
    }

    #[inline(always)]
    fn log2(self) -> f32 {
        libm::log2f(self)
    }

    #[inline(always)]
    fn log10(self) -> f32 {
        libm::log10f(self)
    }

    #[inline(always)]
    fn sin(self) -> f32 {
        libm::sinf(self)
    }

    #[inline(always)]
    fn exp(self) -> f32 {
        libm::expf(self)
    }
}
