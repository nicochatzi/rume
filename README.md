# Rume &emsp; [![Build Status]][actions] [![Latest Version]][crates.io]

[Build Status]: https://img.shields.io/github/workflow/status/nicochatzi/rume/CI/main
[actions]: https://github.com/nicochatzi/rume/actions?query=branch%main
[Latest Version]: https://img.shields.io/crates/v/rume.svg
[crates.io]: https://crates.io/crates/rume

RUst Modular Environment for writing audio DSP graphs.

## Processors

```rust
const TWO_PI: f32 = 2.0 * std::consts::f32::PI;

#[rume::processor]
pub struct Sine {
    phase: f32,
    sample_rate: u32,

    #[rume::processor_input]
    amplitude: f32,

    #[rume::processor_input]
    frequency: f32,

    #[rume::processor_output]
    sample: f32,
}

impl rume::Processor for Sine {
    fn prepare(&mut self, data: rume::AudioConfig) {
        self.sample_rate = data.sample_rate;
    }

    fn process(&mut self) {
        let phase_increment = TWO_PI * self.frequency * (1.0_f32 / self.sample_rate as f32);
        self.phase = (self.phase + phase_increment) % TWO_PI;
        self.sample = self.phase.sin() * self.amplitude;
    }
}
```

## Graphs

```rust
let mut beep = rume::graph! {
    endpoints: {
        audio_out: rume::OutputEndpoint::new(producer),
    },
    processors: {
        freq: rume::Value::new(440.0),
        amp: rume::Value::new(0.8),
        sine: Sine::default(),
    },
    connections: {
        freq.output  ->  sine.input.0,
        amp.output   ->  sine.input.1,
        sine.output  ->  audio_out.input,
    }
};
```

## Endpoints

```rust
// Static lock-free queues
let (mut frequency_producer, frequency_consumer) = rume::input!(FREQUENCY_ENDPOINT);
let (audio_out_producer, mut audio_out_consumer) = rume::output!(AUDIO_OUT_ENDPOINT);

// Input data into the graph from another thread
std::thread::spawn(move || {
    for i in (110..440).step_by(2) {
        frequency_producer.enqueue(i as f32).unwrap();
    }
});

// Extract data from the graph
let spec = hound::WavSpec {
    channels: 1,
    sample_rate: SAMPLE_RATE,
    bits_per_sample: 32,
    sample_format: hound::SampleFormat::Float,
};
let mut writer = hound::WavWriter::create("test.wav", spec).unwrap();

while let Some(sample) = audio_out_consumer.dequeue() {
    writer.write_sample(sample).unwrap();
}

```
