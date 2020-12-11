use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rume::{Processor, Renderable};

pub mod synth {
    rume::graph! {
        inputs: {
            freq: { init: 220.0, range: 64.0..880.0, smooth: 10 },
            amp:  { init:   0.1, range:  0.0..0.8,   smooth: 10 },
        },
        outputs: {
            out,
        },
        processors: {
            sine: rume::Sine::default(),
        },
        connections: {
            freq.output  ->  sine.input.0,
            amp.output   ->  sine.input.1,
            sine.output  ->  out.input,
        }
    }
}

fn main() {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();

    let (mut graph, _, outputs) = synth::build();
    graph.prepare((config.sample_rate().0 as f32).into());

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), graph, outputs).unwrap(),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), graph, outputs).unwrap(),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), graph, outputs).unwrap(),
    }
}

fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut graph: rume::SignalChain,
    mut graph_outputs: synth::Outputs,
) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    let channels = config.channels as usize;

    let mut next_value = move || {
        graph.render(1);
        graph_outputs.out.dequeue().unwrap()
    };

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        |err| eprintln!("an error occurred on stream: {}", err),
    )?;
    stream.play()?;

    std::thread::sleep(std::time::Duration::from_millis(1000));

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
