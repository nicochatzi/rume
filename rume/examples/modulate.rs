#![allow(dead_code)]

use rume::*;

const SAMPLE_RATE: u32 = 44_100;
const BUFFER_SIZE: usize = 16;
const NUM_SECONDS: usize = 10;

const NUM_SAMPLES: usize = SAMPLE_RATE as usize * NUM_SECONDS;
const NUM_BUFFERS: usize = (SAMPLE_RATE as f32 / BUFFER_SIZE as f32) as usize * NUM_SECONDS;
const BUFFER_TIME: f32 = BUFFER_SIZE as f32 / SAMPLE_RATE as f32;

fn main() {
    let (mut frequency_producer, frequency_consumer) = rume::input!(FREQUENCY_ENDPOINT);
    let (audio_out_producer, mut audio_out_consumer) = rume::output!(AUDIO_OUT_ENDPOINT);

    // UI Thread
    std::thread::spawn(move || {
        for i in (110..440).step_by(2) {
            frequency_producer.enqueue(i as f32).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        println!("done modulating");
    });

    // Audio thread
    let _ = std::thread::spawn(move || {
        let mut graph = graph! {
            endpoints: {
                freq: InputEndpoint::new(frequency_consumer),
                audio_out: OutputEndpoint::new(audio_out_producer),
            },
            processors: {
                lvl: Value::new(0.6),
                amt: Value::new(0.1),
                sine: Sine::default(),
                dist: Tanh::default(),
                lpf: Lpf::default(),
            },
            connections: {
                freq.output ->  sine.input.0,
                lvl.output  ->  sine.input.1,
                sine.output ->  dist.input.0,
                amt.output  ->  dist.input.1,
                dist.output ->  lpf.input,
                lpf.output  ->  audio_out.input,
            }
        };

        graph.prepare(SAMPLE_RATE.into());

        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create("test.wav", spec).unwrap();

        for _buffer in 0..NUM_BUFFERS {
            graph.render(BUFFER_SIZE);
            while let Some(sample) = audio_out_consumer.dequeue() {
                writer.write_sample(sample).unwrap();
            }
        }
        println!("done rendering");
    })
    .join();
}

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
