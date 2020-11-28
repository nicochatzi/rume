use crate::{io::*, proc::*};

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
    pub fn render(&mut self, num_samples: usize) {
        for _ in 0..num_samples {
            for processor in self.processors.iter_mut() {
                processor.borrow_mut().process();
                self.connections.transfer(processor.clone());
            }
        }
    }
}

///
unsafe impl Send for SignalChain {}

pub struct SignalChainBuilder {
    chain: SignalChain,
}

impl SignalChainBuilder {
    pub fn new() -> Self {
        Self {
            chain: SignalChain::default(),
        }
    }

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
            .map(|adj_processor| {
                self.chain
                    .processors
                    .index_of(adj_processor.clone())
                    .expect("Could not find processor")
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn empty_chain_does_not_panic() {
        let mut chain = SignalChainBuilder::new().build();
        chain.prepare(48_000.into());
        chain.process();
    }

    struct DummyProcessor {
        input: DummyInput,
        output: DummyOutput,
    }

    impl Processor for DummyProcessor {
        fn prepare(&mut self, _: AudioConfig) {}
        fn process(&mut self) {}
    }

    #[derive(Clone)]
    struct DummyInput;
    impl Input<dyn Processor + 'static> for DummyInput {
        fn set(&self, _: SharedProc<dyn Processor + 'static>, _: f32) {}
    }

    #[derive(Clone)]
    struct DummyOutput;
    impl Output<dyn Processor + 'static> for DummyOutput {
        fn get(&self, _: SharedProc<dyn Processor + 'static>) -> f32 {
            0.0
        }
    }

    fn make_processor() -> Rc<RefCell<DummyProcessor>> {
        Rc::new(RefCell::new(DummyProcessor {
            input: DummyInput,
            output: DummyOutput,
        }))
    }

    #[test]
    fn single_processor_chain_does_not_panic() {
        let mut chain = SignalChainBuilder::new()
            .processor(make_processor())
            .build();
        chain.prepare(48_000.into());
        chain.process();
    }

    #[test]
    fn single_connection_chain_does_not_panic() {
        let dummy = make_processor();
        let mut chain = SignalChainBuilder::new()
            .processor(dummy.clone())
            .connection(
                OutputPort {
                    proc: dummy.clone(),
                    port: Box::new(dummy.clone().borrow().output.clone()),
                },
                InputPort {
                    proc: dummy.clone(),
                    port: Box::new(dummy.clone().borrow().input.clone()),
                },
            )
            .build();
        chain.prepare(48_000.into());
        chain.process();
    }
}
