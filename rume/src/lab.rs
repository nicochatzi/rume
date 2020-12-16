use crate::{OutputStreamConsumer, Processor, Renderable, SignalChain};
use hound::{SampleFormat, WavSpec, WavWriter};
use std::path::PathBuf;

const SAMPLE_RATE: f32 = 48_000.;

pub fn wav_spec() -> WavSpec {
    WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE as u32,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    }
}

///
pub struct GeneratorAnalyzer {
    pub graph: SignalChain,
    pub audio_out: OutputStreamConsumer,
    pub num_samples: u32,
    pub output_path: PathBuf,
}

impl GeneratorAnalyzer {
    pub fn generate(&mut self) -> f32 {
        self.graph.render(1);
        self.audio_out.dequeue().unwrap()
    }

    pub fn wav(&mut self, file_name: &str) {
        let file = self.output_path.join(file_name);
        let mut writer = WavWriter::create(file, wav_spec()).unwrap();

        self.graph.prepare(SAMPLE_RATE.into());
        for _ in 0..self.num_samples {
            writer.write_sample(self.generate()).unwrap();
        }
    }
}
