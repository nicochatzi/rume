#![no_std]

pub use rume_core::*;
pub use rume_macros::*;

#[cfg(target_os = "macos")]
pub mod processors;
#[cfg(target_os = "macos")]
pub use processors::*;
