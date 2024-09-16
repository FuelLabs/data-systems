mod attrs;
mod fields;
mod parse_fn;
mod subject;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Subject, attributes(subject_wildcard, subject_format))]
pub fn subject_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let fields = fields::from_input(&input).unwrap();
    let field_names = fields::names_from_fields(fields);
    let field_types = fields::types_from_fields(fields);
    let parse_fn = parse_fn::create(&input, &field_names);
    let subject_expanded =
        subject::expanded(name, &field_names, &field_types, &input.attrs);

    quote! {
        #subject_expanded

        impl fuel_streams_macros::subject::SubjectBuildable for #name {
            fn new() -> Self {
                Self {
                    #(#field_names: None,)*
                }
            }
        }

        impl IntoSubject for #name {
            #parse_fn
        }

    }
    .into()
}
