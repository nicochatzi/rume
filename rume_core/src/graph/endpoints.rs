use crate::*;
use core::{ops::Range, option::Option};
pub use heapless;
pub use heapless::{
    consts::*,
    spsc::{Consumer, Producer, Queue},
};

pub type StreamDataType = f32;

pub type OutputStreamSize = U2048;
pub type OutputStream = Queue<StreamDataType, OutputStreamSize>;
pub type OutputStreamConsumer = Consumer<'static, StreamDataType, OutputStreamSize>;
pub type OutputStreamProducer = Producer<'static, StreamDataType, OutputStreamSize>;

pub type InputStreamSize = U256;
pub type InputStream = Queue<StreamDataType, InputStreamSize>;
pub type InputStreamConsumer = Consumer<'static, StreamDataType, InputStreamSize>;
pub type InputStreamProducer = Producer<'static, StreamDataType, InputStreamSize>;

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

pub struct InputEndpoint {
    pub output: InputEndpointOutput,
    stream: InputStreamConsumer,
    value: f32,
    range: Option<RangedData>,
    smooth: Option<ValueSmoother>,
    kind: InputEndpointKind,
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
            range: None,
            smooth: None,
            kind: InputEndpointKind::Follow,
        }
    }
}

impl Processor for InputEndpoint {
    fn prepare(&mut self, _: AudioConfig) {
        self.set_value(self.value);
    }

    fn process(&mut self) {
        match self.stream.dequeue() {
            Some(value) => self.set_value(value),
            None => self.process_value(),
        }
    }
}

impl InputEndpoint {
    fn set_value(&mut self, value: f32) {
        let mut new_value = value;

        if let Some(range) = self.range.as_mut() {
            new_value = (*range).clamp(new_value);
        }

        match self.smooth.as_mut() {
            Some(smooth) => smooth.set(self.value, new_value),
            None => self.value = new_value,
        }
    }

    fn process_value(&mut self) {
        match self.kind {
            InputEndpointKind::Trigger => self.value = 0.0,
            InputEndpointKind::Follow => {
                if let Some(smooth) = self.smooth.as_mut() {
                    (*smooth).process(&mut self.value);
                }
            }
        }
    }
}

/// Mainly for internal use to wrap
/// the creation of static data used
/// for endpoints. That data container
/// is then split into a pair:
/// `(producer, consumer)`
///
/// ```
///     use rume_core::{endpoint, InputStream, OutputStream};
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
///     use rume_core::{
///         Output, InputEndpoint, InputEndpointOutput, make_processor, make_input_endpoint, Processor
///     };
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
///     use rume_core::{
///         Input, OutputEndpoint, OutputEndpointInput, make_processor, make_output_endpoint
///     };
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

pub struct InputEndpointBuilder {
    inner: InputEndpoint,
}

impl InputEndpointBuilder {
    pub fn new(stream: InputStreamConsumer) -> Self {
        Self {
            inner: InputEndpoint::new(stream),
        }
    }

    pub fn init(mut self, value: f32) -> Self {
        self.inner.value = value;
        self
    }

    pub fn kind(mut self, kind: InputEndpointKind) -> Self {
        self.inner.kind = kind;
        self
    }

    pub fn range(mut self, range: Range<f32>) -> Self {
        self.inner.range = Some(RangedData::new(range));
        self
    }

    pub fn smooth(mut self, smoothing: u32) -> Self {
        self.inner.smooth = Some(ValueSmoother::new(smoothing));
        self
    }

    pub fn build(self) -> InputEndpoint {
        self.inner
    }
}

pub struct RangedData {
    range: Range<f32>,
}

impl RangedData {
    pub fn new(range: Range<f32>) -> Self {
        Self { range }
    }

    fn clamp(&mut self, value: f32) -> f32 {
        if value > self.range.end {
            self.range.end
        } else if value < self.range.start {
            self.range.start
        } else {
            value
        }
    }
}

#[derive(Debug)]
pub struct ValueSmoother {
    target: f32,
    increment: f32,
    steps: u32,
    step: u32,
}

impl ValueSmoother {
    pub fn new(smoothing: u32) -> Self {
        Self {
            target: 0.0,
            increment: 0.0,
            steps: smoothing,
            step: 0,
        }
    }

    fn set(&mut self, current: f32, target: f32) {
        self.target = target;
        self.increment = (self.target - current) / self.steps as f32;
        self.step = 0;
    }

    fn process(&mut self, value: &mut f32) {
        if self.step < self.steps {
            *value += self.increment;
            self.step += 1;
        } else {
            *value = self.target;
        }
    }
}

#[derive(Debug)]
pub enum InputEndpointKind {
    Follow,
    Trigger,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn input_endpoint_with_init_is_initialised() {
        const INIT_VALUE: f32 = 3.14;

        let (_, consumer) = make_input_endpoint();
        let endpoint = InputEndpointBuilder::new(consumer).init(INIT_VALUE).build();
        let processor = make_processor(endpoint);

        assert_eq!(InputEndpointOutput.get(processor.clone()), INIT_VALUE);
    }

    #[test]
    fn input_endpoint_with_range_is_clamped() {
        const MIN_VALUE: f32 = -1.0;
        const MAX_VALUE: f32 = 10.0;
        const VALUE_IN_RANGE: f32 = 3.14;
        const VALUE_ABOVE_RANGE: f32 = MAX_VALUE + 100.0;
        const VALUE_BELLOW_RANGE: f32 = MIN_VALUE - 10.0;

        let (mut producer, consumer) = make_input_endpoint();
        let processor = make_processor(
            InputEndpointBuilder::new(consumer)
                .range(MIN_VALUE..MAX_VALUE)
                .build(),
        );

        let mut test_value_passing = |value_to_pass: f32, expected: f32| {
            producer.enqueue(value_to_pass).unwrap();
            processor.borrow_mut().process();
            assert_eq!(InputEndpointOutput.get(processor.clone()), expected);
        };

        test_value_passing(VALUE_IN_RANGE, VALUE_IN_RANGE);
        test_value_passing(VALUE_ABOVE_RANGE, MAX_VALUE);
        test_value_passing(VALUE_BELLOW_RANGE, MIN_VALUE);
    }

    #[test]
    fn input_endpoint_with_smoothing_is_smooth() {
        const INIT_VALUE: f32 = -53.1;
        const SMOOTHING: u32 = 10;
        const TARGET_VALUE: f32 = -100.92;

        // This test only considers one direction of
        // movement. But inverting the signs will test
        // the opposite direction: - => down, + => up.
        assert!(INIT_VALUE.abs() < TARGET_VALUE.abs());

        let (mut producer, consumer) = make_input_endpoint();
        let processor = make_processor(
            InputEndpointBuilder::new(consumer)
                .init(INIT_VALUE)
                .smooth(SMOOTHING)
                .build(),
        );

        producer.enqueue(TARGET_VALUE).unwrap();

        processor.borrow_mut().process();
        assert_eq!(InputEndpointOutput.get(processor.clone()), INIT_VALUE);

        for _ in 0..SMOOTHING {
            processor.borrow_mut().process();
            let value = InputEndpointOutput.get(processor.clone());
            assert!(value.abs() < TARGET_VALUE.abs());
            assert!(value.abs() > INIT_VALUE.abs());
        }

        processor.borrow_mut().process();
        assert_eq!(InputEndpointOutput.get(processor.clone()), TARGET_VALUE);
    }

    #[test]
    fn input_endpoint_as_trigger_falls_back_to_zero() {
        const TRIGGER_VALUE: f32 = 1000.0;

        let (mut producer, consumer) = make_input_endpoint();
        let processor = make_processor(
            InputEndpointBuilder::new(consumer)
                .kind(InputEndpointKind::Trigger)
                .build(),
        );

        producer.enqueue(TRIGGER_VALUE).unwrap();

        processor.borrow_mut().process();
        assert_eq!(InputEndpointOutput.get(processor.clone()), TRIGGER_VALUE);

        processor.borrow_mut().process();
        assert_eq!(InputEndpointOutput.get(processor.clone()), 0.0);
    }
}
