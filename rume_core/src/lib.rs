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

#[cfg(not(feature = "std"))]
extern crate alloc;

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

    pub trait F32Ext {
        fn powf(self, exp: f32) -> f32;
        fn log2(self) -> f32;
        fn log10(self) -> f32;
        fn sin(self) -> f32;
        fn exp(self) -> f32;
    }

    impl F32Ext for f32 {
        #[inline]
        fn powf(self, exp: f32) -> f32 {
            libm::powf(self, exp)
        }

        #[inline]
        fn log2(self) -> f32 {
            libm::log2f(self)
        }

        #[inline]
        fn log10(self) -> f32 {
            libm::log10f(self)
        }

        #[inline]
        fn sin(self) -> f32 {
            libm::sinf(self)
        }

        #[inline]
        fn exp(self) -> f32 {
            libm::expf(self)
        }
    }
}
