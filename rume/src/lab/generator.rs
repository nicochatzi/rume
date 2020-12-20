use crate::{
    lab::{plot::plot, spec::AnalyzerSpec},
    InputStreamProducer, OutputStreamConsumer, Processor, Renderable, SignalChain,
};

use std::{fs, io};

use hound::WavWriter;

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

pub enum ModulationRate {
    Audio,
    Control,
}

impl GeneratorAnalyzer {
    /// Render an amount of audio specified by the
    /// `AnalyzerSpec` to a wav file.
    pub fn wav(&mut self, file_name: &str) {
        let mut w = self.spec.wav_writer(file_name);
        self.prepare();
        self.write_wav(&mut w);
    }

    /// Render an amount of audio specified by the
    /// `AnalyzerSpec` to a wav file with input
    /// modulation.
    pub fn wav_with_modulation<F: FnMut()>(
        &mut self,
        file_name: &str,
        mut modulate: F,
        rate: ModulationRate,
    ) {
        let mut w = self.spec.wav_writer(file_name);

        self.prepare();

        for _ in 0..self.spec.num_buffers {
            match rate {
                ModulationRate::Control => {
                    modulate();
                    self.write_wav(&mut w);
                }
                ModulationRate::Audio => {
                    self.write_wav_with_modulation(&mut w, &mut modulate);
                }
            }
        }
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

    fn write_wav(&mut self, w: &mut WavWriter<io::BufWriter<fs::File>>) {
        for _ in 0..self.spec.num_samples {
            w.write_sample(self.model.generate_sample()).unwrap()
        }
    }

    fn write_wav_with_modulation<F: FnMut()>(
        &mut self,
        w: &mut WavWriter<io::BufWriter<fs::File>>,
        modulate: &mut F,
    ) {
        for _ in 0..self.spec.num_samples {
            modulate();
            w.write_sample(self.model.generate_sample()).unwrap()
        }
    }

    fn write_plot(&mut self, file_name: &str) {
        let buffer = self.model.generate_buffer(self.spec.num_samples);
        plot(&buffer, file_name).unwrap();
    }
}
