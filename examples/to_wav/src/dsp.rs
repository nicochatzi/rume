use rume::*;

#[rume::processor]
pub struct Lpf {
    #[rume::processor_input]
    previous: f32,

    #[rume::processor_output]
    current: f32,
}

impl Processor for Lpf {
    fn prepare(&mut self, _: AudioConfig) {}
    fn process(&mut self) {
        self.current = (self.previous + self.current) * 0.5;
        self.previous = self.current;
    }
}

#[rume::processor]
pub struct Tanh {
    #[rume::processor_sample]
    sample: f32,

    #[rume::processor_input]
    amount: f32,
}

impl Processor for Tanh {
    fn prepare(&mut self, _: AudioConfig) {}
    fn process(&mut self) {
        let boost = 4.0;
        if self.amount < 0.1 {
            self.amount = 0.1;
            self.sample = self.amount * (boost * self.sample / self.amount).tanh();
        } else if self.amount > 5.0 {
            self.amount = 5.0;
            self.sample = self.amount * (boost * self.sample / self.amount).tanh();
        } else if self.amount < 1.0 {
            self.sample = (boost * self.sample / self.amount).tanh();
        } else {
            self.sample = self.amount * (boost * self.sample / self.amount).tanh();
        }
    }
}

#[rume::processor]
pub struct ArEnv {
    #[rume::processor_output]
    value: f32,

    attack_ms: f32,
    release_ms: f32,
    tick: u32,
    sample_rate: u32,
}

impl Processor for ArEnv {
    fn prepare(&mut self, config: AudioConfig) {
        self.sample_rate = config.sample_rate;
    }

    fn process(&mut self) {
        let attack_ticks = self.attack_ms / (self.sample_rate as f32 * 1000.0);
        let release_ticks = self.release_ms / (self.sample_rate as f32 * 1000.0);

        if self.tick <= attack_ticks as u32 {
            self.value = 0.0;
        } else if self.tick < release_ticks as u32 {
            self.value = 0.0;
        } else {
            self.tick = 0;
        }
    }
}

#[rume::processor]
pub struct ArEnvCap {
    #[rume::processor_input]
    attack_ms: f32,

    #[rume::processor_input]
    release_ms: f32,

    #[rume::processor_output]
    value: f32,

    tick: u32,
    sample_rate: u32,
}

impl Processor for ArEnvCap {
    fn prepare(&mut self, config: AudioConfig) {
        self.sample_rate = config.sample_rate;
    }

    fn process(&mut self) {
        let attack_ticks = self.attack_ms / (self.sample_rate as f32 * 1000.0);
        let release_ticks = self.release_ms / (self.sample_rate as f32 * 1000.0);

        if self.tick <= attack_ticks as u32 {
            // charge
            self.value = 0.0;
        } else if self.tick < release_ticks as u32 {
            // discharge
            self.value = 0.0;
        } else {
            self.tick = 0;
        }
    }
}
