use proc_macro::TokenStream;

mod graph;
mod io;
mod proc;

#[proc_macro]
pub fn graph(item: TokenStream) -> TokenStream {
    graph::graph(item)
}

#[proc_macro_attribute]
pub fn processor(attr: TokenStream, item: TokenStream) -> TokenStream {
    proc::processor(attr, item)
}

#[proc_macro_attribute]
pub fn processor_output(attr: TokenStream, item: TokenStream) -> TokenStream {
    io::processor_output(attr, item)
}

#[proc_macro_attribute]
pub fn processor_input(attr: TokenStream, item: TokenStream) -> TokenStream {
    io::processor_input(attr, item)
}

// /// can we add this sort of thing to find the name of the obj?
// impl Sine {
//     pub fn inputs(&self, name: &str) -> bool {
//         match name {
//             self._inputs.0.2 => self._inputs.0,
//         }
//     }

//     pub fn outputs(&self, name: &str) -> bool {
//         match name {
//             self._outputs.0.2 => self._outputs.0,
//         }
//     }
// }

// #[rume::processor(SineFrequencyInput)]
// struct Sine {
//     phase: f32,
//     sample_rate: u32,
//     phase_increment: f32,

//     #[rume::processor_input]
//     amplitude: f32,

//     #[rume::processor_output]
//     output: f32,
// }

// should the port have a name to support named inputs that are not linked to a variable here?

// #[rume::processor_input_port(Sine, SineFrequencyInput, "frequency")]
// fn set(&mut self, frequency: f32) {
//     self.phase_increment = TWO_PI * frequency * (1.0_f32 / self.sample_rate as f32);
// }

// impl Processor for Sine {
//     fn prepare(&mut self, data: AudioConfig) {
//         self.sample_rate = data.sample_rate;
//     }

//     fn process(&mut self) {
//         self.phase = (self.phase + self.phase_increment) % TWO_PI;
//         self.sample = self.phase.sin() * self.amplitude;
//     }
// }

// #[rume::processor]
// struct Tanh {
//     #[rume::processor_input]
//     amount: f32,

//     #[rume::processor_input]
//     input: f32,

//     #[rume::processor_output]
//     output: f32,
// }

// impl Processor for Sine {
//     fn prepare(&mut self, _: AudioConfig) {}
//     fn process(&mut self) {
//         self.output = (self.amount * self.input).tanh();
//     }
// }

// #[rume::processor]
// struct ArEnv {
//     sample_rate: u32,
//     tick: u32,

//     #[rume::processor_input(ArEnvAttackInput)]
//     attack_ticks: f32,

//     #[rume::processor_input(ArEnvReleaseInput)]
//     release_ticks: f32,

//     #[rume::processor_output]
//     value: f32,
// }

// #[rume::processor_input_port(ArEnv, ArEnvAttackInput)]
// fn set(&mut self, attack_ms: f32) {
//     self.attack_ticks = attack_ms / (self.sample_rate * 1000.0);
// }

// #[rume::processor_input_port(ArEnv, ArEnvReleaseInput)]
// fn set_release(&mut self, release_ms: f32) {
//     self.release_ticks = release_ms / (self.sample_rate * 1000.0);
// }

// impl Processor for ArEnv {
//     fn prepare(&mut self, _: AudioConfig) {}
//     fn process(&mut self) {
//         if self.tick <= self.attack_ticks {
//             self.value = 0;
//         } else if self.tick < self.release_ticks {
//             self.value = 0;
//         } else {
//             self.tick = 0;
//         }
//     }
// }
