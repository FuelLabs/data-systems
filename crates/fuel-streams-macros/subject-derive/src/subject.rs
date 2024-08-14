use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Type};

fn create_with_methods<'a>(
    field_names: &'a [&'a Ident],
    field_types: &'a [&'a Type],
) -> impl Iterator<Item = TokenStream> + 'a {
    field_names
        .iter()
        .zip(field_types.iter())
        .map(|(name, ty)| {
            let method_name = format_ident!("with_{}", name);
            quote! {
                pub fn #method_name(mut self, value: #ty) -> Self {
                    self.#name = value;
                    self
                }
            }
        })
}

pub fn expanded<'a>(
    name: &'a Ident,
    field_names: &'a [&'a Ident],
    field_types: &'a [&'a syn::Type],
) -> TokenStream {
    let with_methods = create_with_methods(field_names, field_types);
    quote! {
        impl #name {
            pub fn build(
                #(#field_names: #field_types,)*
            ) -> Self {
                Self {
                    #(#field_names,)*
                }
            }

            pub fn wildcard(
                #(#field_names: #field_types,)*
            ) -> String {
                Self::build(#(#field_names,)*).parse()
            }

            #(#with_methods)*
        }

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.parse())
            }
        }
    }
}
