#![allow(dead_code)]

mod dsp;

use crate::dsp::*;
use rume::*;

const SAMPLE_RATE: u32 = 44_100;
const BUFFER_SIZE: usize = 16;
const NUM_SECONDS: usize = 10;

const NUM_SAMPLES: usize = SAMPLE_RATE as usize * NUM_SECONDS;
const NUM_BUFFERS: usize = (SAMPLE_RATE as f32 / BUFFER_SIZE as f32) as usize * NUM_SECONDS;
const BUFFER_TIME: f32 = BUFFER_SIZE as f32 / SAMPLE_RATE as f32;

fn main() {
    let (mut frequency_producer, frequency_consumer) = rume::input!(FREQUENCY_ENDPOINT);
    let (audio_out_producer, mut audio_out_consumer) = rume::output!(AUDIO_OUT_ENDPOINT);

    // UI Thread
    std::thread::spawn(move || {
        for i in (110..440).step_by(2) {
            frequency_producer.enqueue(i as f32).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        println!("done modulating");
    });

    // Audio thread
    let _ = std::thread::spawn(move || {
        let mut graph = graph! {
            endpoints: {
                freq: InputEndpoint::new(frequency_consumer),
                audio_out: OutputEndpoint::new(audio_out_producer),
            },
            processors: {
                lvl: Value::new(0.6),
                amt: Value::new(0.1),
                sine: Sine::default(),
                dist: Tanh::default(),
                lpf: Lpf::default(),
            },
            connections: {
                freq.output ->  sine.input.0,
                lvl.output  ->  sine.input.1,
                sine.output ->  dist.input.0,
                amt.output  ->  dist.input.1,
                dist.output ->  lpf.input,
                lpf.output  ->  audio_out.input,
            }
        };

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
            while let Some(sample) = audio_out_consumer.dequeue() {
                writer.write_sample(sample).unwrap();
            }
        }
        println!("done rendering");
    })
    .join();
}
