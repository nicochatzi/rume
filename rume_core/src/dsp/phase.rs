#[derive(Default, Debug, Clone)]
pub struct Phasor {
    increment: f32,
    accumulator: f32,
    max: f32,
}

impl Phasor {
    #[inline(always)]
    pub fn new(increment: f32, max: f32) -> Self {
        Self {
            accumulator: 0.0,
            increment,
            max,
        }
    }

    #[inline(always)]
    pub fn with_max(max: f32) -> Self {
        Self::new(1.0, max)
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        self.accumulator = 0.0;
    }

    #[inline(always)]
    pub fn inc(&mut self, increment: f32) {
        self.increment = increment;
    }

    #[inline(always)]
    pub fn shift(&mut self, shift: f32) {
        self.accumulator += shift;
        self.accumulator %= self.max;
    }

    #[inline(always)]
    pub fn get(&self) -> f32 {
        self.accumulator
    }

    #[inline(always)]
    pub fn advance(&mut self) -> f32 {
        self.shift(self.increment);
        self.get()
    }
}

#[cfg(test)]
mod test {
    use core::f32::EPSILON;

    use super::*;

    #[test]
    fn wraps_when_passing_max() {
        let mut phasor = Phasor::new(0.5, 2.0);
        assert_eq!(phasor.get(), 0.0);
        assert_eq!(phasor.advance(), 0.5);
        assert_eq!(phasor.advance(), 1.0);
        assert_eq!(phasor.advance(), 1.5);
        assert_eq!(phasor.advance(), 0.0);
    }

    #[test]
    fn resetting_moves_back_to_zero() {
        let mut phasor = Phasor::new(0.1, 1.0);
        assert_eq!(phasor.get(), 0.0);
        assert_eq!(phasor.advance(), 0.1);
        phasor.reset();
        assert_eq!(phasor.get(), 0.0);
    }

    #[test]
    fn shifting_moves_phase_with_wrap() {
        const MAX: f32 = 1.0;
        const INC: f32 = 0.1;
        const SHIFT: f32 = 0.2;

        let mut phasor = Phasor::new(INC, MAX);
        assert_eq!(phasor.get(), 0.0);
        assert_eq!(phasor.advance(), INC);

        phasor.reset();
        phasor.shift(SHIFT);
        assert_eq!(phasor.get(), SHIFT);

        phasor.reset();
        assert_eq!(phasor.get(), 0.0);

        phasor.shift(SHIFT + MAX);
        assert!(phasor.get() > SHIFT - EPSILON);
        assert!(phasor.get() < SHIFT + EPSILON);
    }
}
