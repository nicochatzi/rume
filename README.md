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
pub mod synth {
    rume::graph! {
        inputs: {
            freq: { init: 220.0, range: 64.0..880.0, smooth: 10 },
            amp:  { init:   0.1, range:  0.0..0.8,   smooth: 10 },
        },
        outputs: {
            out,
        },
        processors: {
            sine: rume::Sine::default(),
        },
        connections: {
            freq.output  ->  sine.input.0,
            amp.output   ->  sine.input.1,
            sine.output  ->  out.input,
        }
    }
}

fn main() {
    let (mut graph, inputs, outputs) = synth::build();

    std::thread::spawn(move || {
        for i in (110..440).step_by(2) {
            inputs.freq.enqueue(i as f32).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    std::thread::spawn(move || {
        let config = rume::AudioConfig {
            sample_rate: SAMPLE_RATE,
            buffer_size: BUFFER_SIZE,
            num_channels: 1,
        }
        graph.prepare(config);
        graph.process();

        while let Some(sample) = outputs.out.dequeue() {
            println!("{}", sample);
        }
    }).join();`
}

```