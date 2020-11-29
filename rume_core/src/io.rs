use crate::proc::*;
use std::ops::{Deref, DerefMut};

pub type OwnedDynInput = InputPort<dyn Processor, dyn Input<dyn Processor>>;
pub type OwnedDynOutput = OutputPort<dyn Processor, dyn Output<dyn Processor>>;

pub type OwnedDynConnection =
    Connection<dyn Processor, dyn Input<dyn Processor>, dyn Processor, dyn Output<dyn Processor>>;

pub trait Input<P>
where
    P: Processor + ?Sized,
{
    fn set(&self, proc: SharedProc<P>, data: f32);
}

pub trait Output<P>
where
    P: Processor + ?Sized,
{
    fn get(&self, proc: SharedProc<P>) -> f32;
}

pub struct InputPort<P, I>
where
    P: Processor + ?Sized,
    I: Input<P> + ?Sized,
{
    pub proc: SharedProc<P>,
    pub port: Box<I>,
}

unsafe impl<P, I> Send for InputPort<P, I>
where
    P: Processor + ?Sized,
    I: Input<P> + ?Sized,
{
}

impl<P, I> InputPort<P, I>
where
    P: Processor + ?Sized,
    I: Input<P> + ?Sized,
{
    pub fn set(&mut self, data: f32) {
        self.port.set(self.proc.clone(), data);
    }
}

impl<P, I> PartialEq for InputPort<P, I>
where
    P: Processor + ?Sized,
    I: Input<P> + ?Sized,
{
    fn eq(&self, other: &InputPort<P, I>) -> bool {
        // also check type of inputs
        self.proc.as_ptr() == other.proc.as_ptr()
    }
}

pub struct OutputPort<P, O>
where
    P: Processor + ?Sized,
    O: Output<P> + ?Sized,
{
    pub proc: SharedProc<P>,
    pub port: Box<O>,
}

unsafe impl<P, O> Send for OutputPort<P, O>
where
    P: Processor + ?Sized,
    O: Output<P> + ?Sized,
{
}

impl<P, O> OutputPort<P, O>
where
    P: Processor + ?Sized,
    O: Output<P> + ?Sized,
{
    #[inline(always)]
    pub fn get(&mut self) -> f32 {
        self.port.get(self.proc.clone())
    }
}

impl<P, O> PartialEq for OutputPort<P, O>
where
    P: Processor + ?Sized,
    O: Output<P> + ?Sized,
{
    fn eq(&self, other: &OutputPort<P, O>) -> bool {
        // also check type of outputs
        self.proc.as_ptr() == other.proc.as_ptr()
    }
}

pub struct Connection<PIn, I, POut, O>
where
    PIn: Processor + ?Sized,
    I: Input<PIn> + ?Sized,
    POut: Processor + ?Sized,
    O: Output<POut> + ?Sized,
{
    pub input: InputPort<PIn, I>,
    pub output: OutputPort<POut, O>,
}

impl<PIn, I, POut, O> Connection<PIn, I, POut, O>
where
    PIn: Processor + ?Sized,
    I: Input<PIn> + ?Sized,
    POut: Processor + ?Sized,
    O: Output<POut> + ?Sized,
{
    pub fn transfer(&mut self) {
        self.input.set(self.output.get());
    }

    pub fn new(output: OutputPort<POut, O>, input: InputPort<PIn, I>) -> Self {
        Self { input, output }
    }
}

impl<PIn, I, POut, O> PartialEq for Connection<PIn, I, POut, O>
where
    PIn: Processor + ?Sized,
    I: Input<PIn> + ?Sized,
    POut: Processor + ?Sized,
    O: Output<POut> + ?Sized,
{
    fn eq(&self, other: &Connection<PIn, I, POut, O>) -> bool {
        self.input == other.input && self.output == other.output
    }
}

/// A short-hand for creating a processor
/// input. This creates a zero-sized struct
/// that represents an Input that is linked
/// to a `Processor`.
///
/// ```
///     #[macro_use]
///     use rume_core::*;
///
///     #[derive(Default)]
///     pub struct DummyProcessor {
///         pub input: DummyInput,
///         pub output: (),
///         value: f32,
///     }
///
///     impl Processor for DummyProcessor {
///         fn prepare(&mut self, _: AudioConfig) {}
///         fn process(&mut self) {}
///     }
///
///     input! { DummyProcessor, DummyInput,
///         |proc: &mut DummyProcessor, value: f32| {
///             proc.value = value;
///         }
///     }
/// ```
#[macro_export]
macro_rules! input {
    ($processor:ident, $input_name:ident, $setter:expr) => {
        #[derive(Debug, Default, Clone)]
        pub struct $input_name;
        impl $crate::Input<$crate::DynProc> for $input_name {
            fn set(&self, proc: $crate::SharedProc<$crate::DynProc>, value: f32) {
                let processor = unsafe { &mut (*(proc.as_ptr() as *mut $processor)) };
                ($setter)(processor, value);
            }
        }
    };
}

/// A short-hand for creating a processor
/// output. This creates a zero-sized struct
/// that represents an Output that is linked
/// to a `Processor`.
///
/// ```
///     #[macro_use]
///     use rume_core::*;
///
///     #[derive(Default)]
///     pub struct DummyProcessor {
///         pub input: (),
///         pub output: DummyOutput,
///         value: f32,
///     }
///
///     impl Processor for DummyProcessor {
///         fn prepare(&mut self, _: AudioConfig) {}
///         fn process(&mut self) {}
///     }
///
///     output! { DummyProcessor, DummyOutput,
///         |proc: &mut DummyProcessor| -> f32 {
///             proc.value
///         }
///     }
/// ```
#[macro_export]
macro_rules! output {
    ($processor:ident, $output_name:ident, $getter:expr) => {
        #[derive(Debug, Default, Clone)]
        pub struct $output_name;
        impl $crate::Output<$crate::DynProc> for $output_name {
            fn get(&self, proc: $crate::SharedDynProc) -> f32 {
                let processor = unsafe { &mut (*(proc.as_ptr() as *mut $processor)) };
                ($getter)(processor)
            }
        }
    };
}

/// Macro to create an `OutputPort` in a short and more flexible way.
///
/// It takes an instance of a struct that implements `Processor`.
/// It can optionally take the index of the port,
/// if the processor has multiple ports.
#[macro_export]
macro_rules! make_output_port {
    ($proc:expr $(, $port_num:tt)*) => {
        $crate::OutputPort {
            proc: $proc.clone(),
            port: Box::new($proc.clone().borrow().output $(. $port_num)* .clone()),
        }
    };
}

/// Macro to create an `InputPort` in a short and more flexible way.
///
/// It takes an instance of a struct that implements `Processor`.
/// It can optionally take the index of the port,
/// if the processor has multiple ports.
#[macro_export]
macro_rules! make_input_port {
    ($proc:expr $(, $port_num:tt)*) => {
        $crate::InputPort {
            proc: $proc.clone(),
            port: Box::new($proc.clone().borrow().input $(. $port_num)* .clone()),
        }
    };
}

#[derive(Default)]
pub struct Connections {
    pub inner: Vec<Box<OwnedDynConnection>>,
}

impl Deref for Connections {
    type Target = Vec<Box<OwnedDynConnection>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Connections {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Connections {
    pub fn push(&mut self, connection: Box<OwnedDynConnection>) {
        if let None = self.inner.iter().find(|c| **c == connection) {
            self.inner.push(connection);
        }
    }

    pub fn outputs(&self, processor: SharedDynProc) -> Vec<SharedDynProc> {
        let mut outputs = Vec::<SharedDynProc>::new();
        self.inner
            .iter()
            .filter(|connection| connection.output.proc.as_ptr() == processor.as_ptr())
            .for_each(|connection| outputs.push(connection.input.proc.clone()));
        outputs
    }

    pub fn transfer(&mut self, processor: SharedDynProc) {
        self.inner
            .iter_mut()
            .filter(|connection| connection.output.proc.as_ptr() == processor.as_ptr())
            .for_each(|connection| connection.transfer());
    }
}
