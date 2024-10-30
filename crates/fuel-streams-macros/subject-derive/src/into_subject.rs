use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident};

pub fn parse_fn(input: &DeriveInput, field_names: &[&Ident]) -> TokenStream {
    let format_str = super::attrs::subject_attr("format", &input.attrs);
    let parse_fields = field_names.iter().map(|name| {
        quote! {
            let #name = fuel_streams_macros::subject::parse_param(&self.#name);
        }
    });

    quote! {
        fn parse(&self) -> String {
            #(#parse_fields)*
            format!(#format_str)
        }
    }
}

pub fn wildcard_fn() -> TokenStream {
    quote! {
        fn wildcard(&self) -> &'static str {
           Self::WILDCARD
        }
    }
}
