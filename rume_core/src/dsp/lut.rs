//!
//!
use crate::lib::Vec;
use crate::Phasor;

pub mod interpolate {
    #[inline(always)]
    pub fn linear(x0: f32, x1: f32, w: f32) -> f32 {
        (1_f32 - (w)) * x0 + (w * x1)
    }

    #[inline(always)]
    pub fn lookup(table: &[f32], index: f32) -> f32 {
        let index0: usize = index as usize;
        let index1: usize = (index0 + 1) % table.len();
        let weight: f32 = index - index0 as f32;
        linear(table[index0], table[index1], weight)
    }
}

/// Lookup Table that does not
/// own the data is uses.
#[derive(Default, Debug, Copy, Clone)]
pub struct Lut<'a> {
    pub phasor: Phasor,
    table: &'a [f32],
}

impl<'a> Lut<'a> {
    pub fn new(table: &'a [f32]) -> Self {
        Self {
            phasor: Phasor::with_max(table.len() as f32),
            table,
        }
    }

    pub fn step(&mut self) -> f32 {
        interpolate::lookup(self.table, self.phasor.advance())
    }
}

/// Lookup Table that constructs
/// and owns the table it uses.
#[derive(Default, Debug, Clone)]
pub struct OwnedLut {
    pub phasor: Phasor,
    table: Vec<f32>,
}

impl OwnedLut {
    pub fn new<F: Fn(f32) -> f32>(closure: F, size: usize) -> Self {
        let mut table = Vec::with_capacity(size);
        (0..size).for_each(|i| table.push(closure(i as f32 / size as f32)));
        Self {
            phasor: Phasor::with_max(size as f32),
            table,
        }
    }

    pub fn advance(&mut self) -> f32 {
        interpolate::lookup(&self.table, self.phasor.advance())
    }
}
