use crate::{InputStreamProducer, OutputStreamConsumer, Processor, Renderable, SignalChain};
use hound::{SampleFormat, WavSpec, WavWriter};
use rume_core::AudioConfig;
use std::{fs, io, path::PathBuf};

/// A generic specification for an analysis.
///
#[derive(Debug, Clone)]
pub struct AnalyzerSpec {
    pub sample_rate: f32,
    pub num_samples: u32,
    pub output_path: PathBuf,
    pub num_buffers: Option<u32>,
}

impl Default for AnalyzerSpec {
    fn default() -> Self {
        Self {
            sample_rate: 48_000.,
            num_samples: 512,
            num_buffers: None,
            output_path: PathBuf::new(),
        }
    }
}

impl AnalyzerSpec {
    fn to_config(&self) -> AudioConfig {
        AudioConfig {
            sample_rate: self.sample_rate as usize,
            buffer_size: self.num_samples as usize,
            num_channels: 1,
        }
    }

    fn wav_spec(&self) -> WavSpec {
        WavSpec {
            channels: 1,
            sample_rate: self.sample_rate as u32,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        }
    }

    fn wav_writer(&self, file_name: &str) -> WavWriter<io::BufWriter<fs::File>> {
        let file = self.output_path.join(file_name);
        WavWriter::create(file, self.wav_spec()).unwrap()
    }
}

/// A signal generator data model.
/// Contains a graph that produces audio
/// but does not receive any input.
pub struct GeneratorModel {
    pub graph: SignalChain,
    pub audio_out: OutputStreamConsumer,

    /// An optional "reset button".
    /// If the signal generator needs
    /// to be reset before generation,
    /// provide an trigger InputEndpoint
    /// that clears the graph after one
    /// Processor::process() call.
    pub reset: Option<InputStreamProducer>,
}

impl GeneratorModel {
    pub fn new(
        graph: SignalChain,
        audio_out: OutputStreamConsumer,
        reset: Option<InputStreamProducer>,
    ) -> Self {
        Self {
            graph,
            audio_out,
            reset,
        }
    }

    fn generate_sample(&mut self) -> f32 {
        self.graph.render(1);
        self.audio_out.dequeue().unwrap()
    }
}

/// A signal generator analyzer.
pub struct GeneratorAnalyzer {
    pub model: GeneratorModel,
    pub spec: AnalyzerSpec,
}

impl GeneratorAnalyzer {
    pub fn new(model: GeneratorModel, spec: AnalyzerSpec) -> Self {
        Self { model, spec }
    }

    /// Render an amount of audio specified by the
    /// `AnalyzerSpec` to a wav file.
    pub fn wav(&mut self, file_name: &str) {
        self.prepare();
        self.reset();
        self.write_wav(file_name);
    }

    fn prepare(&mut self) {
        self.model.graph.prepare(self.spec.to_config());
    }

    fn reset(&mut self) {
        if let Some(reset) = self.model.reset.as_mut() {
            reset.enqueue(1.0).unwrap();
            let _ = self.model.generate_sample();
        }
    }

    fn write_wav(&mut self, file_name: &str) {
        let mut writer = self.spec.wav_writer(file_name);
        for _ in 0..self.spec.num_samples {
            writer.write_sample(self.model.generate_sample()).unwrap();
        }
    }
}
