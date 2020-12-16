use std::path::PathBuf;

use rume::lab::GeneratorAnalyzer;

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
    GeneratorAnalyzer {
        graph,
        audio_out: outputs.out,
        num_samples: 512,
        output_path: PathBuf::new(),
    }
    .wav("sine.wav");
}
