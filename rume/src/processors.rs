use core::*;

#[macros::processor]
pub struct Value {
    #[macros::processor_output]
    value: f32,
}

impl Processor for Value {
    fn prepare(&mut self, _: AudioConfig) {}
    fn process(&mut self) {}
}

impl Value {
    pub fn new(value: f32) -> Self {
        let mut v = Self::default();
        v.value = value;
        v
    }
}

#[macros::processor]
pub struct Sine {
    #[macros::processor_input]
    frequency: f32,

    #[macros::processor_input]
    amplitude: f32,

    #[macros::processor_output]
    sample: f32,

    phase: f32,
    sample_period: f32,
}

impl Processor for Sine {
    fn prepare(&mut self, data: AudioConfig) {
        self.sample_period = 1.0 / data.sample_rate;
    }

    fn process(&mut self) {
        const TWO_PI: f32 = 2.0_f32 * std::f32::consts::PI;
        let increment = TWO_PI * self.frequency * self.sample_period;
        self.phase = (self.phase + increment) % TWO_PI;
        self.sample = self.phase.sin() * self.amplitude;
    }
}
