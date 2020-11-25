use std::{
    cell::RefCell,
    f32::consts::PI,
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

pub const TWO_PI: f32 = 2.0_f32 * PI;

#[derive(Clone)]
pub struct AudioConfig {
    pub sample_rate: u32,
}

impl Into<AudioConfig> for u32 {
    fn into(self) -> AudioConfig {
        AudioConfig { sample_rate: self }
    }
}

pub type SharedProc<P> = Rc<RefCell<P>>;
pub type SharedDynProc = SharedProc<dyn Processor>;

pub type SharedProcView<P> = Weak<RefCell<P>>;
pub type SharedDynProcView = SharedProcView<dyn Processor>;

pub type OwnedDynInput = InputPort<dyn Processor, dyn Input<dyn Processor>>;
pub type OwnedDynOutput = OutputPort<dyn Processor, dyn Output<dyn Processor>>;

pub type OwnedDynConnection =
    Connection<dyn Processor, dyn Input<dyn Processor>, dyn Processor, dyn Output<dyn Processor>>;

pub trait Processor {
    fn prepare(&mut self, data: AudioConfig);
    fn process(&mut self);
}

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

#[derive(Default)]
struct Connections {
    inner: Vec<Box<OwnedDynConnection>>,
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
    fn push(&mut self, connection: Box<OwnedDynConnection>) {
        if let None = self.inner.iter().find(|c| **c == connection) {
            self.inner.push(connection);
        }
    }

    fn outputs(&self, processor: SharedDynProc) -> Vec<SharedDynProc> {
        let mut outputs = Vec::<SharedDynProc>::new();
        self.inner
            .iter()
            .filter(|connection| connection.output.proc.as_ptr() == processor.as_ptr())
            .for_each(|connection| outputs.push(connection.input.proc.clone()));
        outputs
    }

    fn transfer(&mut self, processor: SharedDynProc) {
        self.inner
            .iter_mut()
            .filter(|connection| connection.output.proc.as_ptr() == processor.as_ptr())
            .for_each(|connection| connection.transfer());
    }
}

#[derive(Default)]
struct Processors {
    inner: Vec<SharedDynProc>,
}

impl Deref for Processors {
    type Target = Vec<SharedDynProc>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Processors {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Processors {
    fn push(&mut self, processor: SharedDynProc) {
        if self
            .inner
            .iter()
            .filter(|p| Rc::ptr_eq(p, &processor))
            .count()
            == 0
        {
            self.inner.push(processor);
        }
    }

    fn index_of(&self, processor: SharedDynProc) -> usize {
        for (i, candidate) in self.inner.iter().enumerate() {
            if Rc::ptr_eq(candidate, &processor) {
                return i;
            }
        }
        panic!("Could not find processor")
    }

    fn order(&mut self, order: Vec<usize>) {
        for i in 0..(order.len() / 2) {
            self.inner.swap(i, order[i]);
        }
    }
}

#[derive(Default)]
pub struct SignalChain {
    processors: Processors,
    connections: Connections,
}

impl Processor for SignalChain {
    fn prepare(&mut self, config: AudioConfig) {
        self.processors
            .inner
            .iter_mut()
            .for_each(|processor| processor.borrow_mut().prepare(config.clone()));
    }

    fn process(&mut self) {}
}

impl SignalChain {
    pub fn new() -> SignalChainBuilder {
        SignalChainBuilder {
            chain: Self::default(),
        }
    }

    pub fn render(&mut self, num_samples: usize) {
        for _ in 0..num_samples {
            for processor in self.processors.iter_mut() {
                processor.borrow_mut().process();
                self.connections.transfer(processor.clone());
            }
        }
    }
}

unsafe impl Send for SignalChain {}

pub struct SignalChainBuilder {
    chain: SignalChain,
}

impl SignalChainBuilder {
    pub fn processor(mut self, processor: SharedDynProc) -> Self {
        self.chain.processors.push(processor);
        self
    }

    pub fn connection(mut self, output: OwnedDynOutput, input: OwnedDynInput) -> Self {
        self.chain
            .connections
            .push(Box::new(Connection::new(output, input)));
        self
    }

    pub fn build(mut self) -> SignalChain {
        self.sort();
        self.chain
    }

    fn sort_inner(&mut self, index: usize, visited: &mut Vec<bool>, ordering: &mut Vec<usize>) {
        visited[index] = true;

        for i in self.next_processors(index) {
            if !visited[i] {
                self.sort_inner(i, visited, ordering);
            }
        }

        ordering.push(index);
    }

    fn sort(&mut self) {
        let mut ordering = Vec::<usize>::new();
        let mut visited = vec![false; self.chain.processors.inner.len()];

        for i in 0..self.chain.processors.inner.len() {
            if !visited[i] {
                self.sort_inner(i, &mut visited, &mut ordering);
            }
        }

        ordering.reverse();

        self.chain.processors.order(ordering);
    }

    fn next_processors(&self, index: usize) -> Vec<usize> {
        let root_processor = self.chain.processors.inner.get(index).unwrap().clone();
        self.chain
            .connections
            .outputs(root_processor)
            .iter()
            .map(|adj_processor| self.chain.processors.index_of(adj_processor.clone()))
            .collect()
    }
}
