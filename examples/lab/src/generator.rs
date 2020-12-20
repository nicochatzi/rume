use rume::{
    lab::{generator::*, spec::*},
    *,
};

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
        reset: f32,

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
            reset: { kind: trigger },
            freq: { init: 440.0, range: 32.0..880.0 },
            amp: { init: 1.0, range: 0.0..1.0 },
        },
        outputs: { out, },
        processors: { sine: Sine::new(), },
        connections: {
            reset.output -> sine.input.0,
            freq.output  -> sine.input.1,
            amp.output   -> sine.input.2,
            sine.output  -> out.input,
        }
    }
}

pub fn analyze() {
    let (graph, ins, outs) = sine::build();
    let (mut freq, mut amp, reset) = (ins.freq, ins.amp, ins.reset);

    let mut analyzer = GeneratorAnalyzer {
        model: GeneratorModel {
            graph,
            audio_out: outs.out,
            reset: Some(reset),
        },
        spec: AnalyzerSpec::default(),
    };

    analyzer.plot("sine.png");
    analyzer.wav("sine.wav");

    let mut f = 440_f32;
    let mut a = 1_f32;
    analyzer.spec.num_buffers = 8;
    analyzer.wav_with_modulation(
        "sine_modulate.wav",
        move || {
            freq.enqueue(f).unwrap();
            amp.enqueue(a).unwrap();
            f *= 0.999;
            a *= 0.999;
        },
        ModulationRate::Audio,
    );
}
