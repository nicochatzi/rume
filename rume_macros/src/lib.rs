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
