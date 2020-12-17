use crate::{
    AudioConfig, InputStreamProducer, OutputStreamConsumer, Processor, Renderable, SignalChain,
};

use std::{fs, io, path::PathBuf};

use hound::{SampleFormat, WavSpec, WavWriter};
use plotters::prelude::*;

/// A generic specification for an analysis.
///
#[derive(Debug, Clone)]
pub struct AnalyzerSpec {
    pub sample_rate: f32,
    pub num_samples: usize,
    pub output_path: PathBuf,
    pub num_buffers: Option<usize>,
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
    fn generate_sample(&mut self) -> f32 {
        self.graph.render(1);
        self.audio_out.dequeue().unwrap()
    }

    fn generate_buffer(&mut self, num_samples: usize) -> Vec<f32> {
        let mut buffer = Vec::with_capacity(num_samples);
        (0..num_samples).for_each(|_| buffer.push(self.generate_sample()));
        buffer
    }
}

/// A signal generator analyzer.
pub struct GeneratorAnalyzer {
    pub model: GeneratorModel,
    pub spec: AnalyzerSpec,
}

impl GeneratorAnalyzer {
    /// Render an amount of audio specified by the
    /// `AnalyzerSpec` to a wav file.
    pub fn wav(&mut self, file_name: &str) {
        self.prepare();
        self.write_wav(file_name);
    }

    ///
    pub fn plot(&mut self, file_name: &str) {
        self.prepare();
        self.write_plot(file_name);
    }

    fn prepare(&mut self) {
        self.model.graph.prepare(self.spec.to_config());
        self.reset();
    }

    fn reset(&mut self) {
        if let Some(reset) = self.model.reset.as_mut() {
            reset.enqueue(1.0).unwrap();
            let _ = self.model.generate_sample();
        }
    }

    fn write_wav(&mut self, file_name: &str) {
        let mut w = self.spec.wav_writer(file_name);
        (0..self.spec.num_samples)
            .for_each(|_| w.write_sample(self.model.generate_sample()).unwrap());
    }

    fn write_plot(&mut self, file_name: &str) {
        let buffer = self.model.generate_buffer(self.spec.num_samples);
        plot(&buffer, file_name).unwrap();
    }
}

/// https://coolors.co/e87461-fff07c-77bb99-97efe9-0085cc-b497d6-e8eaed-3c4353
pub mod palette {
    use plotters::prelude::*;

    pub const GREEN: RGBColor = RGBColor(119, 187, 153);
    pub const RED: RGBColor = RGBColor(232, 116, 97);
    pub const YELLOW: RGBColor = RGBColor(255, 240, 124);
    pub const LIGHT_B: RGBColor = RGBColor(151, 239, 233);
    pub const DEEP_B: RGBColor = RGBColor(0, 133, 204);
    pub const PURPLE: RGBColor = RGBColor(180, 151, 214);
    pub const GREY: RGBColor = RGBColor(60, 67, 83);
    pub const WHITE: RGBColor = RGBColor(232, 234, 237);
}

pub fn plot(buffer: &[f32], file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let x_range = 0_f64..buffer.len() as f64;
    let y_range = -1.0..1.0;

    let root = BitMapBackend::new(file_name, (1024, 768)).into_drawing_area();
    root.fill(&palette::WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .right_y_label_area_size(40)
        .margin(5)
        .build_cartesian_2d(x_range.clone(), y_range.clone())?
        .set_secondary_coord(x_range, y_range);

    chart
        .configure_mesh()
        .y_label_formatter(&|x| format!("{:e}", x))
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            buffer
                .iter()
                .enumerate()
                .map(|(i, x)| (i as f64, *x as f64)),
            &palette::DEEP_B,
        ))?
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &palette::DEEP_B));

    chart
        .configure_series_labels()
        .background_style(&RGBColor(128, 128, 128))
        .draw()?;

    Ok(())
}
