# Rume &emsp;

[![Build Status]][actions] [![Latest Version]][crates.io] [![codecov]][coverage]

[Build Status]: https://img.shields.io/github/workflow/status/nicochatzi/rume/CI/main
[actions]: https://github.com/nicochatzi/rume/actions?query=branch%main
[Latest Version]: https://img.shields.io/crates/v/rume.svg
[crates.io]: https://crates.io/crates/rume
[codecov]:https://codecov.io/gh/nicochatzi/rume/branch/main/graph/badge.svg
[coverage]:https://codecov.io/gh/nicochatzi/rume

RUst Modular Environment for writing audio DSP graphs.

## Features

By default this crate uses `std`. It supports `#![no_std]` with `default-features = false`.

* `std`: Use the Rust standard library.
* `lab`: (requires std) A set of utilities for analyzing a graph.

## Examples

* [beep](rume/examples/beep.rs): Hello world of audio.
* [modulate](rume/examples/modulate.rs): Multi-threaded modulation.
* [lab](examples/lab/src/main.rs): Graph analysis.

## Usage

### Processor declaration

```rust
#[rume::processor]
pub struct Sine {
    phasor: rume::Phasor,
    sample_time: f32,

    #[rume::input]
    amplitude: f32,

    #[rume::input]
    frequency: f32,

    #[rume::output]
    sample: f32,
}

impl rume::Processor for Sine {
    fn prepare(&mut self, data: rume::AudioConfig) {
        self.sample_time = 1. / data.sample_rate as f32;
    }

    fn process(&mut self) {
        const TWO_PI: f32 = 2.0 * std::consts::f32::PI;
        self.phasor.inc(TWO_PI * self.frequency * self.sample_time));
        self.sample = self.phasor.advance().sin() * self.amplitude;
    }
}
```

### Graph declaration

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
```

### Graph usage

```rust
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
            sample_rate: 48_000_f32.into(),
            buffer_size: 512,
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