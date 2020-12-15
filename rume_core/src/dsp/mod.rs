//! A module that encapsulates a set of DSP
//! utilities as well as a set of core processors,

pub mod lut;
pub use lut::*;

pub mod osc;
pub use osc::*;

pub mod phase;
pub use phase::*;

pub mod convert;
pub mod waves;
