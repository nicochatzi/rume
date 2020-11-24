use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn};

pub fn processor_output(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut attributes = attr.into_iter();

    let processor_type = format_ident!(
        "{}",
        attributes
            .nth(0)
            .expect("First attribute should be the processor name")
            .to_string()
    );

    let _consume = attributes.nth(0);

    let output_type = format_ident!(
        "{}",
        attributes
            .nth(0)
            .expect("Second attribute should be the output struct name")
            .to_string()
    );

    let function_name = format_ident!(
        "{}",
        item.clone()
            .into_iter()
            .nth(1)
            .expect("The supplied getter method should not take any other keyword than fn")
            .to_string()
    );

    let function_definition = parse_macro_input!(item as ItemFn);

    (quote! {
        #[derive(Debug, Default, Clone)]
        pub struct #output_type;
        impl Output<dyn Processor + 'static> for #output_type {
            fn get(&self, this: SharedProc<dyn Processor + 'static>) -> f32 {
                let processor = unsafe {&mut (*(this.as_ptr() as *mut #processor_type)) };
                #[inline(always)]
                #function_definition;
                #function_name(processor)
            }
        }
    })
    .into()
}

pub fn processor_input(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut attributes = attr.into_iter();

    let processor_type = format_ident!(
        "{}",
        attributes
            .nth(0)
            .expect("First attribute should be the processor name")
            .to_string()
    );

    let _consume = attributes.nth(0);

    let input_type = format_ident!(
        "{}",
        attributes
            .nth(0)
            .expect("Second attribute should be the input struct name")
            .to_string()
    );

    let function_name = format_ident!(
        "{}",
        item.clone()
            .into_iter()
            .nth(1)
            .expect("The supplied setter method should not take any other keyword than fn")
            .to_string()
    );

    let function_definition = parse_macro_input!(item as ItemFn);

    (quote! {
        #[derive(Debug, Default, Clone)]
        pub struct #input_type;
        impl Input<dyn Processor + 'static> for #input_type {
            fn set(&self, this: SharedProc<dyn Processor + 'static>, value: f32) {
                let processor = unsafe {&mut (*(this.as_ptr() as *mut #processor_type)) };
                #[inline(always)]
                #function_definition;
                #function_name(processor, value);
            }
        }
    })
    .into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
