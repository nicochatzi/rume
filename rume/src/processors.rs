use crate::*;

#[processor]
pub struct Value {
    #[output]
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

mod table {
    pub const SIZE: usize = 256;
    pub const FREQ: f32 = 48_000. / SIZE as f32;
    pub const TIME: f32 = 1. / FREQ;
}

#[processor]
pub struct Sine {
    #[input]
    frequency: f32,

    #[input]
    amplitude: f32,

    #[output]
    sample: f32,

    lut: OwnedLut,
    sample_time: f32,
}

impl Sine {
    pub fn new() -> Self {
        let mut sine = Self::default();
        sine.lut = OwnedLut::new(|x: f32| (x * 2. * core::f32::consts::PI).sin(), table::SIZE);
        sine
    }
}

impl Processor for Sine {
    fn prepare(&mut self, config: AudioConfig) {
        self.sample_time = 1.0 / config.sample_rate as f32;
    }

    fn process(&mut self) {
        self.lut.phasor.increment = self.frequency * table::TIME;
        self.sample = self.lut.advance() * self.amplitude;
    }
}

#[processor]
pub struct Saw {
    #[input]
    frequency: f32,

    #[input]
    amplitude: f32,

    #[output]
    sample: f32,

    phasor: Phasor,
    sample_time: f32,
}

impl Processor for Saw {
    fn prepare(&mut self, config: AudioConfig) {
        self.sample_time = 1.0 / config.sample_rate as f32;
        self.phasor.reset();
    }

    fn process(&mut self) {
        let cps = self.frequency * self.sample_time;
        self.phasor.increment = cps;
        let phase = self.phasor.advance();
        self.sample = waves::saw::rise(phase);
        self.sample -= bandlimited::step((phase + 0.5) % 1., cps);
        self.sample *= self.amplitude;
    }
}
