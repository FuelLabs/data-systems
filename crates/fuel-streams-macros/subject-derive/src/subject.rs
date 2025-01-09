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

fn create_get_methods<'a>(
    field_names: &'a [&'a Ident],
    field_types: &'a [&'a Type],
) -> impl Iterator<Item = TokenStream> + 'a {
    field_names
        .iter()
        .zip(field_types.iter())
        .map(|(name, ty)| {
            let method_name = format_ident!("get_{}", name);
            quote! {
                pub fn #method_name(&self) -> &#ty {
                    &self.#name
                }
            }
        })
}

pub fn expanded<'a>(
    name: &'a Ident,
    field_names: &'a [&'a Ident],
    field_types: &'a [&'a syn::Type],
    attrs: &'a [syn::Attribute],
) -> TokenStream {
    let with_methods = create_with_methods(field_names, field_types);
    let get_methods = create_get_methods(field_names, field_types);
    let wildcard = crate::attrs::subject_attr("wildcard", attrs);

    quote! {
        impl #name {
            pub const WILDCARD: &'static str = #wildcard;

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

            pub fn boxed(self) -> Box<Self> {
                Box::new(self)
            }

            pub fn arc(self) -> std::sync::Arc<Self> {
                std::sync::Arc::new(self)
            }

            #(#with_methods)*
            #(#get_methods)*
        }

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.parse())
            }
        }
    }
}
