use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

#[derive(Clone)]
pub struct AudioConfig {
    pub sample_rate: f32,
}

impl Into<AudioConfig> for u32 {
    fn into(self) -> AudioConfig {
        AudioConfig {
            sample_rate: self as f32,
        }
    }
}

impl Into<AudioConfig> for f32 {
    fn into(self) -> AudioConfig {
        AudioConfig { sample_rate: self }
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
    fn prepare(&mut self, data: AudioConfig);
    fn process(&mut self);
}

#[derive(Default)]
pub struct Processors {
    pub inner: Vec<SharedDynProc>,
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
        for i in 0..(order.len() / 2) {
            self.inner.swap(i, order[i]);
        }
    }
}
