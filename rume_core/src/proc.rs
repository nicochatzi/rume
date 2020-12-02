use crate::io::*;
use core::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};
use std::rc::{Rc, Weak};

#[derive(Clone, Copy)]
pub struct AudioConfig {
    pub sample_rate: usize,
    pub buffer_size: usize,
    pub num_channels: usize,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48_000,
            buffer_size: 64,
            num_channels: 2,
        }
    }
}

impl Into<AudioConfig> for u32 {
    fn into(self) -> AudioConfig {
        let mut config = AudioConfig::default();
        config.sample_rate = self as usize;
        config
    }
}

impl Into<AudioConfig> for f32 {
    fn into(self) -> AudioConfig {
        let mut config = AudioConfig::default();
        config.sample_rate = self as usize;
        config
    }
}

/// A shared pointer to a generic type
/// which will be expected to be a `Processor`.
pub type SharedProc<P> = Rc<RefCell<P>>;

/// A generic dynamic Processor.
pub type DynProc = dyn Processor + 'static;

/// A shared pointer to a dynamic `Processor`.
pub type SharedDynProc = SharedProc<DynProc>;

/// A weak pointer, i.e. a "view" of
/// a generic type.
pub type SharedProcView<P> = Weak<RefCell<P>>;

/// A weak pointer to a dynamic `Processor`.
pub type SharedDynProcView = SharedProcView<DynProc>;

pub trait Processor {
    fn prepare(&mut self, config: AudioConfig);
    fn process(&mut self);
}

pub fn make_processor<P>(processor: P) -> SharedProc<P> {
    Rc::new(RefCell::new(processor))
}

/// A `Processor` with views
/// to its inputs and outputs.
/// This is effectively a node
/// in the linked-list.
pub struct ConnectedProcessor {
    proc: SharedDynProc,
    outs: Vec<DynConnection>,
}

impl ConnectedProcessor {
    pub fn new(proc: SharedDynProc) -> Self {
        Self {
            proc,
            outs: Vec::new(),
        }
    }

    pub fn add_output(&mut self, connection: DynConnection) {
        if self.outs.iter().find(|c| **c == connection).is_none() {
            self.outs.push(connection);
        }
    }

    pub fn outs(&self) -> &[DynConnection] {
        self.outs.as_slice()
    }

    pub fn outs_mut(&mut self) -> &mut [DynConnection] {
        self.outs.as_mut_slice()
    }
}

impl Processor for ConnectedProcessor {
    fn prepare(&mut self, config: AudioConfig) {
        self.proc.borrow_mut().prepare(config);
    }

    fn process(&mut self) {
        self.proc.borrow_mut().process();
        self.outs.iter_mut().for_each(|con| con.transfer());
    }
}

/// A wrapper around a list
/// of processors. This is
/// effectively the linked-list.
#[derive(Default)]
pub struct ConnectedProcessors {
    inner: Vec<ConnectedProcessor>,
}

impl ConnectedProcessors {
    pub fn push(&mut self, processor: SharedDynProc) {
        if self
            .inner
            .iter()
            .filter(|p| Rc::ptr_eq(&p.proc, &processor))
            .count()
            == 0
        {
            self.inner.push(ConnectedProcessor::new(processor));
        }
    }

    pub fn index_of(&self, processor: SharedDynProc) -> Option<usize> {
        for (i, candidate) in self.inner.iter().enumerate() {
            if Rc::ptr_eq(&candidate.proc, &processor) {
                return Some(i);
            }
        }
        None
    }

    pub fn order(&mut self, order: Vec<usize>) {
        for (i, j) in order.iter().enumerate().take(order.len() / 2) {
            self.inner.swap(i, *j);
        }
    }

    pub fn find(&self, processor: SharedDynProc) -> Option<&ConnectedProcessor> {
        let idx = self.index_of(processor)?;
        Some(&self.inner[idx])
    }

    pub fn find_mut(&mut self, processor: SharedDynProc) -> Option<&mut ConnectedProcessor> {
        let idx = self.index_of(processor)?;
        Some(&mut self.inner[idx])
    }
}

impl Processor for ConnectedProcessors {
    fn prepare(&mut self, config: AudioConfig) {
        self.inner.iter_mut().for_each(|proc| proc.prepare(config));
    }

    fn process(&mut self) {
        self.inner.iter_mut().for_each(|proc| proc.process());
    }
}

impl Deref for ConnectedProcessors {
    type Target = Vec<ConnectedProcessor>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ConnectedProcessors {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[cfg(test)]
pub mod dummies {
    use super::*;

    #[derive(Default)]
    pub struct DummyProcessor {
        pub input: DummyInput,
        pub output: DummyOutput,
        value: f32,
    }

    impl Processor for DummyProcessor {
        fn prepare(&mut self, _: AudioConfig) {}
        fn process(&mut self) {}
    }

    input! { DummyProcessor, DummyInput,
        |proc: &mut DummyProcessor, value: f32| {
            proc.value = value;
        }
    }

    output! { DummyProcessor, DummyOutput,
        |proc: &mut DummyProcessor| -> f32 {
            proc.value
        }
    }

    pub fn dummy() -> SharedProc<DummyProcessor> {
        make_processor(DummyProcessor::default())
    }

    #[derive(Default)]
    pub struct MultiInProcessor {
        pub input: (MultiInInput, MultiInInput, MultiInInput),
        pub output: MultiInOutput,
        value: f32,
    }

    impl Processor for MultiInProcessor {
        fn prepare(&mut self, _: AudioConfig) {}
        fn process(&mut self) {}
    }

    input! { MultiInProcessor, MultiInInput,
        |proc: &mut MultiInProcessor, value: f32| {
            proc.value = value;
        }
    }

    output! { MultiInProcessor, MultiInOutput,
        |proc: &mut MultiInProcessor| -> f32 {
            proc.value
        }
    }

    #[derive(Default)]
    pub struct MultiOutProcessor {
        pub input: MultiOutInput,
        pub output: (MultiOutOutput, MultiOutOutput, MultiOutOutput),
        value: f32,
    }

    impl Processor for MultiOutProcessor {
        fn prepare(&mut self, _: AudioConfig) {}
        fn process(&mut self) {}
    }

    input! { MultiOutProcessor, MultiOutInput,
        |proc: &mut MultiOutProcessor, value: f32| {
            proc.value = value;
        }
    }

    output! { MultiOutProcessor, MultiOutOutput,
        |proc: &mut MultiOutProcessor| -> f32 {
            proc.value
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Default)]
    struct Proc {}
    impl Processor for Proc {
        fn prepare(&mut self, _: AudioConfig) {}
        fn process(&mut self) {}
    }

    fn dummy() -> SharedDynProc {
        make_processor(Proc::default())
    }

    #[test]
    fn pushing_same_proc_only_adds_the_first() {
        let (a, b) = (dummy(), dummy());
        let mut processors = ConnectedProcessors::default();
        processors.push(a.clone());
        processors.push(a.clone());
        processors.push(b.clone());

        assert_eq!(processors.len(), 2);
        assert_eq!(processors.index_of(a).unwrap(), 0);
        assert_eq!(processors.index_of(b).unwrap(), 1);
    }

    #[test]
    fn reordering_processors_swaps_their_positions() {
        const NUM_PROCESSORS: usize = 6;

        let mut dummies = Vec::<SharedDynProc>::with_capacity(NUM_PROCESSORS);
        let mut processors = ConnectedProcessors::default();

        for _ in 0..NUM_PROCESSORS {
            let d = dummy();
            dummies.push(d.clone());
            processors.push(d.clone())
        }

        for (i, p) in processors.iter().enumerate() {
            assert_eq!(processors.index_of(p.proc.clone()), Some(i))
        }

        processors.swap(0, 3);
        processors.swap(1, 5);
        processors.swap(2, 4);

        assert_eq!(processors.len(), NUM_PROCESSORS);
        assert_eq!(processors.index_of(dummies[0].clone()), Some(3));
        assert_eq!(processors.index_of(dummies[1].clone()), Some(5));
        assert_eq!(processors.index_of(dummies[2].clone()), Some(4));
    }
}
