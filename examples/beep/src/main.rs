use std::f32::consts::PI;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub const TWO_PI: f32 = 2.0_f32 * PI;

#[rume::processor]
pub struct Sine {
    phase: f32,
    sample_rate: u32,

    #[rume::processor_input]
    amplitude: f32,

    #[rume::processor_input]
    frequency: f32,

    #[rume::processor_output]
    sample: f32,
}

impl rume::Processor for Sine {
    fn prepare(&mut self, data: rume::AudioConfig) {
        self.sample_rate = data.sample_rate;
    }

    fn process(&mut self) {
        let phase_increment = TWO_PI * self.frequency * (1.0_f32 / self.sample_rate as f32);
        self.phase = (self.phase + phase_increment) % TWO_PI;
        self.sample = self.phase.sin() * self.amplitude;
    }
}

fn main() {
    let (producer, mut consumer) = rume::output!(AUDIO_OUT_ENDPOINT);

    let mut beep = rume::graph! {
        endpoints: {
            audio_out: rume::OutputEndpoint::new(producer),
        },
        processors: {
            freq: rume::Value::new(440.0),
            amp: rume::Value::new(0.8),
            sine: Sine::default(),
        },
        connections: {
            freq.output  ->  sine.input.0,
            amp.output   ->  sine.input.1,
            sine.output  ->  audio_out.input,
        }
    };

    // let host = cpal::default_host();

    // let device = host
    //     .default_output_device()
    //     .expect("failed to find a default output device");
    // let config = device.default_output_config().unwrap();

    // match config.sample_format() {
    //     cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), beep, consumer).unwrap(),
    //     cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), beep, consumer).unwrap(),
    //     cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), beep, consumer).unwrap(),
    // }
}

// fn run<T>(
//     device: &cpal::Device,
//     config: &cpal::StreamConfig,
//     graph: rume::SignalChain,
//     consumer: rume::OutputStreamConsumer,
// ) -> Result<(), anyhow::Error>
// where
//     T: cpal::Sample,
// {
//     let channels = config.channels as usize;

//     graph.prepare(config.sample_rate.0.into());

//     let mut next_value = move || {
//         graph.render(1);
//         consumer.dequeue().unwrap()
//     };

//     let stream = device.build_output_stream(
//         config,
//         move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
//             write_data(data, channels, &mut next_value)
//         },
//         |err| eprintln!("an error occurred on stream: {}", err),
//     )?;
//     stream.play()?;

//     std::thread::sleep(std::time::Duration::from_millis(1000));

//     Ok(())
// }

// fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
// where
//     T: cpal::Sample,
// {
//     for frame in output.chunks_mut(channels) {
//         let value: T = cpal::Sample::from::<f32>(&next_sample());
//         for sample in frame.iter_mut() {
//             *sample = value;
//         }
//     }
// }
