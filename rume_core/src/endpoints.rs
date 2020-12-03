use crate::*;
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

#[macro_export]
macro_rules! input_endpoint {
    ($endpoint_name:ident) => {{
        static mut $endpoint_name: $crate::InputStream =
            heapless::spsc::Queue(heapless::i::Queue::new());
        unsafe { $endpoint_name.split() }
    }};
}

#[macro_export]
macro_rules! output_endpoint {
    ($endpoint_name:ident) => {{
        static mut $endpoint_name: $crate::OutputStream =
            heapless::spsc::Queue(heapless::i::Queue::new());
        unsafe { $endpoint_name.split() }
    }};
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn input_endpoint_consumes_data() {
        const VALUE_TO_PASS: f32 = 3.14;
        let (mut producer, consumer) = input_endpoint!(DUMMY_ENDPOINT);
        let processor = make_processor(InputEndpoint::new(consumer));

        producer.enqueue(VALUE_TO_PASS).unwrap();
        processor.borrow_mut().process();

        assert_eq!(InputEndpointOutput.get(processor.clone()), VALUE_TO_PASS);
    }

    #[test]
    fn output_endpoint_produces_data() {
        const VALUE_TO_PASS: f32 = 3.14;
        let (producer, mut consumer) = output_endpoint!(DUMMY_ENDPOINT);
        let processor = make_processor(OutputEndpoint::new(producer));

        OutputEndpointInput.set(processor.clone(), VALUE_TO_PASS);
        assert_eq!(consumer.dequeue().unwrap(), VALUE_TO_PASS);
    }
}
