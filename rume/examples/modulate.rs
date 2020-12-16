use std::{thread, time::Duration};

use rume::*;

const SAMPLE_RATE: u32 = 44_100;
const BUFFER_SIZE: usize = 16;
const NUM_SECONDS: usize = 10;
const NUM_BUFFERS: usize = (SAMPLE_RATE as f32 / BUFFER_SIZE as f32) as usize * NUM_SECONDS;

pub mod synth {
    rume::graph! {
        inputs: { freq: { init: 220.0 }, amp: { init: 1.0 }, },
        outputs: { audio_out, },
        processors: { sine: rume::Sine::new(), },
        connections: {
            freq.output -> sine.input.0,
            amp.output  -> sine.input.1,
            sine.output -> audio_out.input,
        }
    }
}

fn main() {
    let (mut graph, mut params, mut outs) = synth::build();

    // UI Thread
    thread::spawn(move || {
        for i in (110..440).step_by(2) {
            params.freq.enqueue(i as f32).unwrap();
            thread::sleep(Duration::from_millis(5));
        }
        println!("done modulating");
    });

    // Audio thread
    let _ = thread::spawn(move || {
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
