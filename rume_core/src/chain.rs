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

#[macro_export]
macro_rules! connect {
    ($builder:expr, ($out_proc:expr $(, $out_port:tt)*) => ($in_proc:expr $(, $in_port:tt)*)) => {
        $builder = $builder
            .processor($out_proc.clone())
            .processor($in_proc.clone())
            .connection(
                $crate::make_output_port!($out_proc $(, $out_port)*),
                $crate::make_input_port!($in_proc $(, $in_port)*),
            );
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_chain_does_not_panic() {
        let mut chain = SignalChainBuilder::new().build();
        chain.prepare(48_000.into());
        chain.process();
    }

    #[derive(Default)]
    struct DummyProcessor {
        input: DummyInput,
        output: DummyOutput,
        value: f32,
    }

    impl Processor for DummyProcessor {
        fn prepare(&mut self, _: AudioConfig) {}
        fn process(&mut self) {}
    }

    #[derive(Default, Clone)]
    struct DummyInput;
    impl Input<dyn Processor + 'static> for DummyInput {
        fn set(&self, this: SharedDynProc, value: f32) {
            let processor = unsafe { &mut (*(this.as_ptr() as *mut DummyProcessor)) };
            processor.value = value
        }
    }

    #[derive(Default, Clone)]
    struct DummyOutput;
    impl Output<dyn Processor + 'static> for DummyOutput {
        fn get(&self, this: SharedDynProc) -> f32 {
            let processor = unsafe { &mut (*(this.as_ptr() as *mut DummyProcessor)) };
            processor.value
        }
    }

    fn make_processor() -> SharedProc<DummyProcessor> {
        make_processor!(DummyProcessor::default())
    }

    #[test]
    fn single_processor_chain_does_not_panic() {
        let mut chain = SignalChainBuilder::new()
            .processor(make_processor())
            .build();
        chain.prepare(48_000.into());
        chain.process();
    }

    fn make_chain(num_processors: usize) -> (SignalChain, Vec<SharedProc<DummyProcessor>>) {
        let processors = vec![make_processor(); num_processors];
        let mut builder = SignalChainBuilder::new();

        for i in 0..processors.len() - 1 {
            connect!(builder, (processors[i], output) => (processors[i + 1], input));
        }

        (builder.build(), processors)
    }

    #[test]
    fn single_connection_chain_does_not_panic() {
        let (mut chain, _) = make_chain(1);
        chain.prepare(48_000.into());
        chain.process();
    }

    #[test]
    fn data_is_passed_through_the_signal_chain() {
        const VALUE_TO_PASS: f32 = 1.0;
        const NUM_PROCESSORS: usize = 30;

        let (mut chain, processors) = make_chain(NUM_PROCESSORS);
        chain.prepare(48_000.into());

        for proc in &processors {
            let out = DummyOutput;
            assert_eq!(out.get(proc.clone()), f32::default());
        }

        let input = DummyInput;
        input.set(processors[0].clone(), VALUE_TO_PASS);

        chain.render(1);

        for proc in processors {
            let out = DummyOutput;
            assert_eq!(out.get(proc), VALUE_TO_PASS);
        }
    }

    #[test]
    fn data_injection_mid_chain_is_overwritten_by_first_input() {
        const VALUE_TO_PASS: f32 = 1.0;
        const VALUE_TO_INJECT: f32 = 2.0;
        const NUM_PROCESSORS: usize = 20;

        let (mut chain, processors) = make_chain(NUM_PROCESSORS);
        chain.prepare(48_000.into());

        for proc in &processors {
            let out = DummyOutput;
            assert_eq!(out.get(proc.clone()), f32::default());
        }

        let input = DummyInput;
        input.set(processors[2].clone(), VALUE_TO_INJECT);

        let input = DummyInput;
        input.set(processors[0].clone(), VALUE_TO_PASS);

        chain.render(1);

        for proc in processors {
            let out = DummyOutput;
            assert_eq!(out.get(proc), VALUE_TO_PASS);
        }
    }

    #[test]
    fn processors_added_in_reverse_order_are_sorted() {
        const VALUE_TO_PASS: f32 = 1.0;
        const NUM_PROCESSORS: usize = 20;

        let processors = vec![make_processor(); NUM_PROCESSORS];
        let mut builder = SignalChainBuilder::new();

        let input = DummyInput;
        input.set(processors[0].clone(), VALUE_TO_PASS);

        for i in 0..processors.len() - 1 {
            connect!(builder, (processors[i], output) => (processors[i + 1], input));
        }
        let mut chain = builder.build();

        chain.prepare(48_000.into());
        chain.render(1);

        for proc in processors {
            let out = DummyOutput;
            assert_eq!(out.get(proc), VALUE_TO_PASS);
        }
    }

    #[derive(Default)]
    struct MultiInProcessor {
        input: (DummyInput, DummyInput, DummyInput),
        output: DummyOutput,
        value: f32,
    }

    impl Processor for MultiInProcessor {
        fn prepare(&mut self, _: AudioConfig) {}
        fn process(&mut self) {}
    }

    #[derive(Default)]
    struct MultiOutProcessor {
        input: DummyInput,
        output: (DummyOutput, DummyOutput, DummyOutput),
        value: f32,
    }

    impl Processor for MultiOutProcessor {
        fn prepare(&mut self, _: AudioConfig) {}
        fn process(&mut self) {}
    }

    #[test]
    fn processor_with_multiple_inputs_gets_sorted() {
        const VALUE_TO_PASS: f32 = 1.0;
        const NUM_PROCESSORS: usize = 20;

        let multi_in = make_processor!(MultiInProcessor::default());
        let multi_out = make_processor!(MultiOutProcessor::default());
        let mut builder = SignalChainBuilder::new();
        connect!(builder, (multi_out, output, 0) => (multi_in, input, 0));
        connect!(builder, (multi_out, output, 1) => (multi_in, input, 1));
        connect!(builder, (multi_out, output, 2) => (multi_in, input, 2));
        let mut chain = builder.build();

        chain.prepare(48_000.into());
        chain.render(1);
    }
}
