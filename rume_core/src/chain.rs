use crate::{io::*, proc::*, sort::*};
use alloc::vec::Vec;

pub trait Renderable {
    fn render(&mut self, num_samples: usize);
}

#[derive(Default)]
pub struct SignalChain {
    processors: ConnectedProcessors,
    config: AudioConfig,
}

unsafe impl Send for SignalChain {}

impl Processor for SignalChain {
    fn prepare(&mut self, config: AudioConfig) {
        self.config.buffer_size = config.buffer_size;
        self.processors.prepare(config);
    }

    fn process(&mut self) {
        self.render(self.config.buffer_size);
    }
}

impl Renderable for SignalChain {
    fn render(&mut self, num_samples: usize) {
        (0..num_samples).for_each(|_| self.processors.process());
    }
}

#[derive(Default)]
pub struct SignalChainBuilder {
    chain: SignalChain,
}

impl SignalChainBuilder {
    pub fn processor(mut self, processor: SharedDynProc) -> Self {
        self.chain.processors.push(processor);
        self
    }

    pub fn connection(mut self, output: DynOutputPort, input: DynInputPort) -> Self {
        self.chain
            .processors
            .find_mut(output.proc.clone())
            .expect("Did not find this output processor in the chain")
            .add_output(Connection::new(output, input));
        self
    }

    pub fn build(mut self) -> SignalChain {
        self.sort();
        self.chain
    }

    fn sort(&mut self) {
        self.chain
            .processors
            .order(TopologicalSort::reverse_sort(&self));
    }
}

impl Sortable for &mut SignalChainBuilder {
    fn next_nodes(&self, index: usize) -> Vec<usize> {
        self.chain
            .processors
            .get(index)
            .unwrap()
            .outs()
            .iter()
            .map(|con| {
                self.chain
                    .processors
                    .index_of(con.output.proc.clone())
                    .expect("Did not find this output processor in the chain")
            })
            .collect()
    }

    fn num_nodes(&self) -> usize {
        self.chain.processors.len()
    }
}

/// A short hand for adding a connection to
/// a `SignalChainBuilder` and implicitly
/// add processor.
#[macro_export]
macro_rules! connect {
    ($builder:expr, ($out_proc:expr $(, $out_port_num:tt)*) => ($in_proc:expr $(, $in_port_num:tt)*)) => {
        $builder = $builder
            .processor($out_proc.clone())
            .processor($in_proc.clone())
            .connection(
                $crate::make_output_port!($out_proc $(, $out_port_num)*),
                $crate::make_input_port!($in_proc $(, $in_port_num)*),
            );
    };
}

/// A short hand for creating a signal chain.
/// This macro takes a series of output-to-input
/// connections and constructs a `SignalChain`.
#[macro_export]
macro_rules! chain {
    ( $( ($out_proc:expr $(, $out_port_num:tt)*) => ($in_proc:expr $(, $in_port_num:tt)*)),* ) => {{
        let mut builder = SignalChainBuilder::default();
        $(
            connect!(builder, ($out_proc $(, $out_port_num)*) => ($in_proc $(, $in_port_num)*));
        )*
        builder.build()
    }};
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::proc::dummies::*;

    #[test]
    fn empty_chain_does_not_panic() {
        let mut chain = SignalChainBuilder::default().build();
        chain.prepare(48_000.into());
        chain.process();
    }

    #[test]
    fn single_processor_chain_does_not_panic() {
        let mut chain = SignalChainBuilder::default().processor(dummy()).build();
        chain.prepare(48_000.into());
        chain.process();
    }

    fn make_chain(num_processors: usize) -> (SignalChain, Vec<SharedProc<DummyProcessor>>) {
        let processors = vec![dummy(); num_processors];
        let mut builder = SignalChainBuilder::default();

        for i in 0..processors.len() - 1 {
            connect!(builder, (processors[i]) => (processors[i + 1]));
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

        let processors = vec![dummy(); NUM_PROCESSORS];
        let mut builder = SignalChainBuilder::default();

        DummyInput.set(processors[0].clone(), VALUE_TO_PASS);

        for i in 0..processors.len() - 1 {
            connect!(builder, (processors[i]) => (processors[i + 1]));
        }
        let mut chain = builder.build();

        chain.prepare(48_000.into());
        chain.render(1);

        for proc in processors {
            assert_eq!(DummyOutput.get(proc), VALUE_TO_PASS);
        }
    }

    #[test]
    fn multiple_io_unsorted_gets_sorted() {
        const VALUE_TO_PASS: f32 = 1.0;

        let (input, mid, output) = (dummy(), dummy(), dummy());
        let multi_in = make_processor(MultiInProcessor::default());
        let multi_out = make_processor(MultiOutProcessor::default());

        let mut chain = chain! {
            (input) => (multi_out),
                       (multi_out, 0)  => (multi_in, 0),
                       (multi_out, 1)  => (mid),
                       (multi_out, 2)  => (multi_in, 2),
                                          (multi_in) => (output)
        };

        chain.prepare(48_000.into());

        DummyInput.set(input.clone(), VALUE_TO_PASS);

        chain.render(1);

        assert_eq!(DummyOutput.get(output), VALUE_TO_PASS);
        assert_eq!(DummyOutput.get(mid), VALUE_TO_PASS);
    }
}
