#[derive(Debug, Clone)]
pub struct Phasor {
    accumulator: f32,
    increment: f32,
    max: f32,
}

impl Default for Phasor {
    fn default() -> Self {
        Self {
            accumulator: 0.0,
            increment: 0.0,
            max: 1.0,
        }
    }
}

impl Phasor {
    pub fn with_max(max: f32) -> Self {
        Self {
            accumulator: 0.0,
            increment: 1.0,
            max,
        }
    }

    pub fn set_increment(&mut self, increment: f32) {
        self.increment = increment;
    }

    pub fn reset(&mut self) {
        self.accumulator = 0.0;
    }

    pub fn shift(&mut self, shift: f32) {
        self.accumulator += shift;
        self.accumulator %= self.max;
    }

    pub fn advance(&mut self) -> f32 {
        self.shift(self.increment);
        self.accumulator
    }
}
