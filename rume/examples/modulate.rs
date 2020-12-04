#![allow(dead_code)]

use rume::*;

const SAMPLE_RATE: u32 = 44_100;
const BUFFER_SIZE: usize = 16;
const NUM_SECONDS: usize = 10;

const NUM_SAMPLES: usize = SAMPLE_RATE as usize * NUM_SECONDS;
const NUM_BUFFERS: usize = (SAMPLE_RATE as f32 / BUFFER_SIZE as f32) as usize * NUM_SECONDS;
const BUFFER_TIME: f32 = BUFFER_SIZE as f32 / SAMPLE_RATE as f32;

#[rume::processor]
pub struct Lpf {
    #[input]
    previous: f32,

    #[output]
    current: f32,
}

impl Processor for Lpf {
    fn prepare(&mut self, _: AudioConfig) {}
    fn process(&mut self) {
        self.current = (self.previous + self.current) * 0.5;
        self.previous = self.current;
    }
}

pub mod synth {
    use super::*;

    rume::graph! {
        inputs: {
            freq,
        },
        outputs: {
            audio_out,
        },
        processors: {
            lvl: rume::Value::new(0.6),
            amt: rume::Value::new(0.1),
            sine: rume::Sine::default(),
            lpf: Lpf::default(),
        },
        connections: {
            freq.output ->  sine.input.0,
            lvl.output  ->  sine.input.1,
            sine.output ->  lpf.input,
            lpf.output  ->  audio_out.input,
        }
    }
}

fn main() {
    let (mut graph, mut params, mut outs) = synth::make();

    // UI Thread
    std::thread::spawn(move || {
        for i in (110..440).step_by(2) {
            params.freq.enqueue(i as f32).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        println!("done modulating");
    });

    // Audio thread
    let _ = std::thread::spawn(move || {
        graph.prepare(SAMPLE_RATE.into());

        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create("test.wav", spec).unwrap();

        for _buffer in 0..NUM_BUFFERS {
            graph.render(BUFFER_SIZE);
            while let Some(sample) = outs.audio_out.dequeue() {
                writer.write_sample(sample).unwrap();
            }
        }
        println!("done rendering");
    })
    .join();
}
