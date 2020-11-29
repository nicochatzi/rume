use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Field};

pub fn processor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(item as DeriveInput);
    let processor = &derive_input.ident;
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
    let parse_io = |field: &mut Field, enum_suffix: &str| {
        field.attrs.clear();
        let mut enum_name = field.clone().ident.unwrap().to_string();
        enum_name.retain(|c| c != '_');
        (
            format_ident!("{}{}{}", name, enum_name, enum_suffix),
            field.clone().ident.unwrap(),
        )
    };

    let mut inputs = Vec::<syn::Ident>::new();
    let mut outputs = inputs.clone();
    let mut input_enums = inputs.clone();
    let mut output_enums = inputs.clone();

    for field in struct_fields.iter_mut() {
        if !field.attrs.is_empty() {
            let attribute_name = field.attrs[0].path.segments[1].ident.clone();

            if attribute_name == "processor_input" || attribute_name == "processor_sample" {
                let (input_enum, input) = parse_io(field, "Input");
                input_enums.push(input_enum);
                inputs.push(input);
            }
            if attribute_name == "processor_output" || attribute_name == "processor_sample" {
                let (output_enum, output) = parse_io(field, "Output");
                output_enums.push(output_enum);
                outputs.push(output)
            }
        }
        fields.push(field.clone());
    }

    (quote! {
        #[derive(Debug, Default, Clone)]
        pub struct #processor {
            pub input: (#(#input_enums),*),
            pub output: (#(#output_enums),*),
            #(#fields),*
        }
        #(
            input! { #processor, #input_enums,
                |proc: &mut #processor, value: f32| {
                    proc.#inputs = value;
                }
            }
        )*
        #(
            output! { #processor, #output_enums,
                |proc: &mut #processor| -> f32 {
                    proc.#outputs
                }
            }
        )*
    })
    .into()
}
