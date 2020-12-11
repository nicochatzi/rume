use proc_macro::TokenStream;

mod graph;
mod proc;

#[proc_macro]
pub fn graph(input: TokenStream) -> TokenStream {
    graph::graph(input)
}

#[proc_macro_attribute]
pub fn processor(attr: TokenStream, item: TokenStream) -> TokenStream {
    proc::processor(attr, item)
}
