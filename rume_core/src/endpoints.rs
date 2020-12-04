use crate::*;
pub use heapless;
pub use heapless::{
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
    value: f32,
}

output! { InputEndpoint, InputEndpointOutput,
    |proc: &mut InputEndpoint| -> f32 {
        proc.value
    }
}

impl InputEndpoint {
    pub fn new(stream: InputStreamConsumer) -> Self {
        Self {
            output: InputEndpointOutput,
            stream,
            value: 0.0,
        }
    }
}

impl Processor for InputEndpoint {
    fn prepare(&mut self, _: AudioConfig) {}
    fn process(&mut self) {
        if let Some(value) = self.stream.dequeue() {
            self.value = value;
        }
    }
}

pub struct OutputEndpoint {
    pub input: OutputEndpointInput,
    stream: OutputStreamProducer,
}

input! { OutputEndpoint, OutputEndpointInput,
    |proc: &mut OutputEndpoint, value: f32| {
        proc.stream.enqueue(value).unwrap();
    }
}

impl OutputEndpoint {
    pub fn new(stream: OutputStreamProducer) -> Self {
        Self {
            input: OutputEndpointInput,
            stream,
        }
    }
}

impl Processor for OutputEndpoint {
    fn prepare(&mut self, _: AudioConfig) {}
    fn process(&mut self) {}
}

/// Mainly for internal use to wrap
/// the creation of static data used
/// for endpoints. That data container
/// is then split into a pair:
/// `(producer, consumer)`
///
/// ```
///     use rume_core::*;
///
///     const VALUE_TO_PASS: f32 = 3.14;
///
///     let (in_producer, in_consumer) = endpoint!(InputStream);
///     let (out_producer, out_consumer) = endpoint!(OutputStream);
///
///     in_producer.enqueue(VALUE_TO_PASS).unwrap();
///     let value = in_consumer.dequeue().unwrap();
///     out_producer.enqueue(value).unwrap();
///
///     assert_eq!(out_consumer.dequeue().unwrap(), VALUE_TO_PASS);
///
/// ```
macro_rules! endpoint {
    ($endpoint_type: ty) => {{
        use $crate::heapless::{i, spsc};
        static mut ENDPOINT: $endpoint_type = spsc::Queue(i::Queue::new());
        unsafe { ENDPOINT.split() }
    }};
}

/// Create an input endpoint producer and consumer.
///
/// ```
///     use rume_core::*;
///
///     const VALUE_TO_PASS: f32 = 3.14;
///
///     let (mut producer, consumer) = make_input_endpoint();
///     let processor = make_processor(InputEndpoint::new(consumer));
///
///     producer.enqueue(VALUE_TO_PASS).unwrap();
///     processor.borrow_mut().process();
///
///     assert_eq!(InputEndpointOutput.get(processor.clone()), VALUE_TO_PASS);
/// ```
pub fn make_input_endpoint() -> (InputStreamProducer, InputStreamConsumer) {
    endpoint!(InputStream)
}

/// Create an output endpoint producer and consumer.
///
/// ```
///     use rume_core::*;
///
///     const VALUE_TO_PASS: f32 = 3.14;
///
///     let (producer, mut consumer) = make_output_endpoint();
///     let processor = make_processor(OutputEndpoint::new(producer));
///
///     OutputEndpointInput.set(processor.clone(), VALUE_TO_PASS);
///     assert_eq!(consumer.dequeue().unwrap(), VALUE_TO_PASS);
/// ```
pub fn make_output_endpoint() -> (OutputStreamProducer, OutputStreamConsumer) {
    endpoint!(OutputStream)
}
