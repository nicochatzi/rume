use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

pub fn processor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(item as DeriveInput);
    let ident = &derive_input.ident;
    let name = derive_input.ident.to_string();

    let struct_fields = match derive_input.data {
        syn::Data::Struct(struct_data) => struct_data.fields,
        _ => panic!("Expected a struct after this declaration that contains fields"),
    };

    let mut struct_fields = match struct_fields {
        syn::Fields::Named(fields) => fields.named,
        _ => panic!("Expected named fields in this struct"),
    };

    let mut fields = Vec::<syn::Field>::new();

    let mut input_enums = Vec::<syn::Ident>::new();
    let mut inputs = Vec::<syn::Ident>::new();

    let mut output_enums = Vec::<syn::Ident>::new();
    let mut outputs = Vec::<syn::Ident>::new();

    for field in struct_fields.iter_mut() {
        if !field.attrs.is_empty() {
            let attribute_name = field.attrs[0].path.segments[1].ident.clone();

            if attribute_name == "processor_input" || attribute_name == "processor_sample" {
                field.attrs.clear();
                let mut enum_name = field.clone().ident.unwrap().to_string();
                enum_name.retain(|c| c != '_');
                input_enums.push(format_ident!("{}{}Input", name, enum_name));
                inputs.push(field.clone().ident.unwrap());
            }
            if attribute_name == "processor_output" || attribute_name == "processor_sample" {
                field.attrs.clear();
                let mut enum_name = field.clone().ident.unwrap().to_string();
                enum_name.retain(|c| c != '_');
                output_enums.push(format_ident!("{}{}Output", name, enum_name));
                outputs.push(field.clone().ident.unwrap());
            }
        }
        fields.push(field.clone());
    }

    // println!("{:#?}", input_enums);
    // println!("{:#?}", inputs);
    // println!("{:#?}", output_enums);
    // println!("{:#?}", outputs);

    let token = quote! {
        #[derive(Debug, Default, Clone)]
        pub struct #ident {
            pub input: (#(#input_enums),*),
            pub output: (#(#output_enums),*),
            #(#fields),*
        }
        #(
            #[derive(Debug, Default, Clone)]
            pub struct #input_enums;
            impl Input<dyn Processor + 'static> for #input_enums {
                fn set(&self, this: SharedProc<dyn Processor + 'static>, value: f32) {
                    let processor = unsafe {&mut (*(this.as_ptr() as *mut #ident)) };
                    processor.#inputs = value;
                }
            }
        )*
        #(
            #[derive(Debug, Default, Clone)]
            pub struct #output_enums;
            impl Output<dyn Processor + 'static> for #output_enums {
                fn get(&self, this: SharedProc<dyn Processor + 'static>) -> f32 {
                    let processor = unsafe {&mut (*(this.as_ptr() as *mut #ident)) };
                    processor.#outputs
                }
            }
        )*
    };
    // println!("{}", token);
    token.into()
}

// let input_i = (0..inputs.len()).map(syn::Index::from);
// let output_i = (0..outputs.len()).map(syn::Index::from);

// let get_input_port_macro = format_ident!("{}__get_input_port", name);
// let get_output_port_macro = format_ident!("{}__get_output_port", name);

// macro_rules! #get_input_port_macro {
//     ($name:expr) => {
//         match $name {
//             #(#inputs => { format!("input.{}", #input_i) } ,)*
//             _ => panic!("Not input named {} for processor {}", name, #name),
//         }
//     };
// }
// macro_rules! #get_output_port_macro {
//     ($name:expr) => {
//         match name {
//             #(#outputs => { format!("output.{}", #output_i) }, )*
//             _ => panic!("Not output named {} for processor {}", name, #name),
//         }
//     };
// }
