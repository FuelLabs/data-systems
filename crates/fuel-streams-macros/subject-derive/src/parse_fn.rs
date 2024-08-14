use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident};

pub fn create(input: &DeriveInput, field_names: &[&Ident]) -> TokenStream {
    let format_str = super::attrs::subject_attr("format", &input.attrs);
    let parse_fields = field_names.iter().map(|name| {
        quote! {
            let #name = Self::parse_param(&self.#name);
        }
    });

    quote! {
        fn parse(&self) -> String {
            #(#parse_fields)*
            format!(#format_str)
        }
    }
}
