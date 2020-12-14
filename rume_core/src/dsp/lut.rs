//!
//!

#[inline(always)]
pub fn lerp(x0: f32, x1: f32, w: f32) -> f32 {
    (1 as f32 - (w)) * x0 + (w * x1)
}

/// table must be power of 2
#[inline(always)]
pub fn filut(table: &[f32], index: f32) -> f32 {
    let length: usize = table.len() - 1;
    let index0: usize = index as usize;
    let index1: usize = (index0 + 1) % length;
    let weight: f32 = index - index0 as f32;
    lerp(table[index0], table[index1], weight)
}

#[derive(Debug, Clone)]
pub struct Phasor {
    accumulator: f32,
    increment: f32,
    max: f32,
}

impl Default for Phasor {
    fn default() -> Phasor {
        Phasor {
            accumulator: 0.0,
            increment: 0.1,
            max: 1.0,
        }
    }
}

impl Phasor {
    pub fn with_max(max: f32) -> Self {
        let mut phasor = Phasor::default();
        phasor.max = max;
        phasor
    }

    pub fn set_increment(&mut self, increment: f32) {
        self.increment = increment;
    }

    pub fn reset(&mut self) {
        self.accumulator = 0.0;
    }

    pub fn step(&mut self) -> f32 {
        self.accumulator += self.increment;
        self.accumulator %= self.max;
        self.accumulator
    }
}

/// Lookup Table that does not
/// own the data is uses.
#[derive(Default, Debug, Clone)]
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
        filut(self.table, self.phasor.step())
    }
}

/// Lookup Table that constructs
/// and owns the table it uses.
#[derive(Debug, Clone)]
pub struct OwnedLut<const N: usize> {
    pub phasor: Phasor,
    table: [f32; N],
}

impl<const N: usize> Default for OwnedLut<N> {
    fn default() -> Self {
        Self {
            phasor: Phasor::with_max(N as f32),
            table: [0.0; N],
        }
    }
}

impl<const N: usize> OwnedLut<N> {
    pub fn new<F: Fn(f32) -> f32>(closure: F) -> Self {
        let mut lut = OwnedLut::default();
        (0..N).for_each(|i| lut.table[i] = closure(i as f32 / N as f32));
        lut
    }

    pub fn step(&mut self) -> f32 {
        filut(&self.table, self.phasor.step())
    }
}
