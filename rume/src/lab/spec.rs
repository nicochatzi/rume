use crate::AudioConfig;

use std::{fs, io, path::PathBuf};

use hound::{SampleFormat, WavSpec, WavWriter};

/// A generic specification for an analysis.
///
#[derive(Debug, Clone)]
pub struct AnalyzerSpec {
    pub sample_rate: f32,
    pub num_samples: usize,
    pub output_path: PathBuf,
    pub num_buffers: usize,
}

impl Default for AnalyzerSpec {
    fn default() -> Self {
        Self {
            sample_rate: 48_000.,
            num_samples: 512,
            num_buffers: 1,
            output_path: PathBuf::new(),
        }
    }
}

impl AnalyzerSpec {
    pub fn to_config(&self) -> AudioConfig {
        AudioConfig {
            sample_rate: self.sample_rate as usize,
            buffer_size: self.num_samples as usize,
            num_channels: 1,
        }
    }

    pub fn wav_spec(&self) -> WavSpec {
        WavSpec {
            channels: 1,
            sample_rate: self.sample_rate as u32,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        }
    }

    pub fn wav_writer(&self, file_name: &str) -> WavWriter<io::BufWriter<fs::File>> {
        let file = self.output_path.join(file_name);
        WavWriter::create(file, self.wav_spec()).unwrap()
    }
}
