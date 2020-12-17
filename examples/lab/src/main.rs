use rume::lab::{AnalyzerSpec, GeneratorAnalyzer, GeneratorModel};

pub mod sine {
    rume::graph! {
        inputs: { freq: { init: 220.0 }, amp: { init: 1.0 }, },
        outputs: { out, },
        processors: { sine: rume::Sine::new(), },
        connections: {
            freq.output -> sine.input.0,
            amp.output  -> sine.input.1,
            sine.output -> out.input,
        }
    }
}

fn main() {
    let (graph, _, outputs) = sine::build();
    let mut analyzer = GeneratorAnalyzer {
        model: GeneratorModel {
            graph,
            audio_out: outputs.out,
            reset: None,
        },
        spec: AnalyzerSpec::default(),
    };

    analyzer.wav("sine.wav");
    analyzer.plot("sine.png");
}
