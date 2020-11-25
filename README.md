# Rume &emsp; [![Build Status]][actions] [![Latest Version]][crates.io] [![codecov]][coverage]

[Build Status]: https://img.shields.io/github/workflow/status/nicochatzi/rume/CI/main
[actions]: https://github.com/nicochatzi/rume/actions?query=branch%main
[Latest Version]: https://img.shields.io/crates/v/rume.svg
[crates.io]: https://crates.io/crates/rume
[codecov]:https://codecov.io/gh/nicochatzi/rume/branch/main/graph/badge.svg
[coverage]:https://codecov.io/gh/nicochatzi/rume

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

std::thread::spawn(move || {
    for i in (110..440).step_by(2) {
        // Input data into the graph from another thread
        frequency_producer.enqueue(i as f32).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
});

std::thread::spawn(move || {
    graph.prepare(SAMPLE_RATE.into());
    graph.render(BUFFER_SIZE);

    // Render and extract samples on another thread
    while let Some(sample) = audio_out_consumer.dequeue() {
        println!("{}", sample);
    }
}).join();
```