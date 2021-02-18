use rume::{
    lab::{processor::*, spec::*},
    *,
};

pub mod sine {
    use super::*;

    #[processor]
    pub struct SimpleLpf {
        #[input]
        audio_in: f32,

        #[output]
        audio_out: f32,
    }

    #[processor]
    pub struct Lpf {
        #[input]
        reset: f32,

        #[input]
        previous: f32,

        #[output]
        current: f32,
    }

    impl Processor for Lpf {
        fn prepare(&mut self, _: AudioConfig) {}
        fn process(&mut self) {
            if self.reset != 0. {
                self.current = 0.;
                self.previous = 0.;
            }

            self.current = (self.previous + self.current) * 0.5;
            self.previous = self.current;
        }
    }

    rume::graph! {
        inputs: {
            audio_in: { init: 0.0 },
            reset: { kind: trigger },
        },
        outputs: { audio_out, },
        processors: { lpf: Lpf::default(), },
        connections: {
            reset.output    -> lpf.input.0,
            audio_in.output -> lpf.input.1,
            lpf.output      -> audio_out.input,
        }
    }
}

pub fn analyze() {
    let (graph, ins, outs) = sine::build();

    let _ = ProcessorAnalyzer {
        model: ProcessorModel {
            graph,
            audio_in: ins.audio_in,
            audio_out: outs.audio_out,
            reset: Some(ins.reset),
        },
        spec: AnalyzerSpec::default(),
    };
}
