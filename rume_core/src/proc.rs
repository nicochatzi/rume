use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

#[derive(Clone)]
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
pub type SharedDynProc = Rc<RefCell<DynProc>>;

/// A weak pointer, i.e. a "view" of
/// a generic type.
pub type SharedProcView<P> = Weak<RefCell<P>>;

/// A weak pointer to a dynamic `Processor`.
pub type SharedDynProcView = Weak<RefCell<DynProc>>;

pub trait Processor {
    fn prepare(&mut self, config: AudioConfig);
    fn process(&mut self);
}

#[derive(Default)]
pub struct Processors {
    inner: Vec<SharedDynProc>,
}

pub fn make_processor<P>(processor: P) -> SharedProc<P> {
    Rc::new(RefCell::new(processor))
}

impl Processors {
    pub fn push(&mut self, processor: SharedDynProc) {
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

    pub fn index_of(&self, processor: SharedDynProc) -> Option<usize> {
        for (i, candidate) in self.inner.iter().enumerate() {
            if Rc::ptr_eq(candidate, &processor) {
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
    fn pushing_same_proc_twice_only_adds_the_first() {
        let (a, b) = (dummy(), dummy());
        let mut processors = Processors::default();
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

        let dummies = vec![dummy(); NUM_PROCESSORS];
        let mut processors = Processors::default();
        dummies.iter().for_each(|p| processors.push(p.clone()));

        for (i, p) in processors.iter().enumerate() {
            assert_eq!(processors.index_of(p.clone()), Some(i))
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
