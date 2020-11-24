use core::*;
use macros::*;

use heapless::{
    consts::*,
    spsc::{Consumer, Producer, Queue},
};

pub type StreamDataType = f32;

pub type InputStreamSize = U256;
pub type InputStream = Queue<StreamDataType, InputStreamSize>;
pub type InputStreamConsumer = Consumer<'static, StreamDataType, InputStreamSize>;
pub type InputStreamProducer = Producer<'static, StreamDataType, InputStreamSize>;

pub type OutputStreamSize = U2048;
pub type OutputStream = Queue<StreamDataType, OutputStreamSize>;
pub type OutputStreamConsumer = Consumer<'static, StreamDataType, OutputStreamSize>;
pub type OutputStreamProducer = Producer<'static, StreamDataType, OutputStreamSize>;

pub struct InputEndpoint {
    pub output: InputEndpointOutput,
    stream: InputStreamConsumer,
    memory: f32,
}

impl InputEndpoint {
    pub fn new(stream: InputStreamConsumer) -> Self {
        Self {
            output: (InputEndpointOutput),
            stream,
            memory: 0.0,
        }
    }
}

impl Processor for InputEndpoint {
    fn prepare(&mut self, _: AudioConfig) {}
    fn process(&mut self) {
        if let Some(value) = self.stream.dequeue() {
            self.memory = value;
        }
    }
}

#[processor_output(InputEndpoint, InputEndpointOutput)]
fn get(input: &mut InputEndpoint) -> f32 {
    input.memory
}

pub struct OutputEndpoint {
    pub input: OutputEndpointInput,
    stream: OutputStreamProducer,
}

#[processor_input(OutputEndpoint, OutputEndpointInput)]
fn set(output: &mut OutputEndpoint, value: f32) {
    output.stream.enqueue(value).unwrap();
}

impl OutputEndpoint {
    pub fn new(stream: OutputStreamProducer) -> Self {
        Self {
            input: (OutputEndpointInput),
            stream,
        }
    }
}

impl Processor for OutputEndpoint {
    fn prepare(&mut self, _: AudioConfig) {}
    fn process(&mut self) {}
}

// struct Endpoints {
//     inputs: Vec<InputEndpoint>,
//     outputs: Vec<OutputEndpoint>,
// }

// struct Module {
//     input: (ModuleInput0, ModuleInput1),
//     output: (ModuleOutput0),
//     endpoints: Endpoints,
//     chain: SignalChain,
// }

// impl Processor for Module {
//     fn prepare(&mut self, config: AudioConfig) {
//         self.chain.prepare(config);
//     }

//     fn process(&mut self) {
//         self.chain.process();
//     }
// }

// impl Module {
//     pub fn new(chain: SignalChain, endpoints: Endpoints) -> Self {
//         Self {
//             input: (ModuleInput0, ModuleInput1),
//             output: (ModuleOutput0),
//             endpoints,
//             chain,
//         }
//     }
// }

// #[processor_input(Module, ModuleInput0)]
// fn set(module: &mut Module, value: f32) {
//     module.endpoints.inputs.get(0).unwrap().set(value)
// }

// #[processor_input(Module, ModuleInput1)]
// fn set(module: &mut Module, value: f32) {}

// #[processor_output(Module, ModuleOutput0)]
// fn get(module: &mut Module) -> f32 {}

#[macro_export]
macro_rules! input {
    ($endpoint_name:ident) => {{
        static mut $endpoint_name: rume::InputStream =
            heapless::spsc::Queue(heapless::i::Queue::new());
        unsafe { $endpoint_name.split() }
    }};
}

#[macro_export]
macro_rules! output {
    ($endpoint_name:ident) => {{
        static mut $endpoint_name: rume::OutputStream =
            heapless::spsc::Queue(heapless::i::Queue::new());
        unsafe { $endpoint_name.split() }
    }};
}
