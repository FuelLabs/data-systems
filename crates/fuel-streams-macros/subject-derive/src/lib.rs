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
    let wildcard = attrs::subject_attr("wildcard", &input.attrs);
    let parse_fn = parse_fn::create(&input, &field_names);
    let subject_expanded = subject::expanded(name, &field_names, &field_types);

    quote! {
        #subject_expanded

        impl IntoSubject for #name {
            const WILDCARD: &'static str = #wildcard;
            #parse_fn

            fn new() -> Self {
                Self {
                    #(#field_names: None,)*
                }
            }
        }
    }
    .into()
}
