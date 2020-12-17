use rume::{lab::*, *};

pub mod sine {
    use super::*;

    const TWO_PI: f32 = 2. * core::f32::consts::PI;

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

        #[input]
        reset: f32,

        #[output]
        sample: f32,

        lut: OwnedLut,
        sample_time: f32,
    }

    impl Sine {
        pub fn new() -> Self {
            let mut sine = Self::default();
            sine.lut = OwnedLut::new(|x: f32| (x * TWO_PI).sin(), table::SIZE);
            sine
        }
    }

    impl Processor for Sine {
        fn prepare(&mut self, config: AudioConfig) {
            self.sample_time = 1.0 / config.sample_rate as f32;
        }

        fn process(&mut self) {
            if self.reset != 0. {
                self.lut.phasor.reset();
            }

            self.lut.phasor.inc(self.frequency * table::TIME);
            self.sample = self.lut.advance() * self.amplitude;
        }
    }

    rume::graph! {
        inputs: {
            freq: { init: 220.0 },
            amp: { init: 1.0 },
            reset: { kind: trigger },
        },
        outputs: { out, },
        processors: { sine: Sine::new(), },
        connections: {
            freq.output  -> sine.input.0,
            amp.output   -> sine.input.1,
            reset.output -> sine.input.2,
            sine.output  -> out.input,
        }
    }
}

fn main() {
    let (graph, inputs, outputs) = sine::build();
    let mut analyzer = GeneratorAnalyzer {
        model: GeneratorModel {
            graph,
            audio_out: outputs.out,
            reset: Some(inputs.reset),
        },
        spec: AnalyzerSpec::default(),
    };

    analyzer.plot("sine.png");
    analyzer.wav("sine.wav");
}
