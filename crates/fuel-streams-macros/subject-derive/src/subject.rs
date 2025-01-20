use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Type};

use crate::attrs::SubjectAttrs;

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
    let query_all = crate::attrs::subject_attr("query_all", attrs);
    let id = crate::attrs::subject_attr("id", attrs);
    let entity = crate::attrs::subject_attr("entity", attrs);

    // Get custom_where if it exists, otherwise use None
    let custom_where = if let Some(extra) =
        SubjectAttrs::from_attributes(attrs).get("custom_where")
    {
        quote! { Some(#extra) }
    } else {
        quote! { None }
    };

    quote! {
        impl #name {
            pub const ID: &'static str = #id;
            pub const QUERY_ALL: &'static str = #query_all;
            pub const ENTITY: &'static str = #entity;
            pub const EXTRA_WHERE: Option<&'static str> = #custom_where;

            pub fn build(
                #(#field_names: #field_types,)*
            ) -> Self {
                Self {
                    #(#field_names,)*
                }
            }

            pub fn build_string(
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

            pub fn dyn_arc(self) -> std::sync::Arc<dyn IntoSubject> {
                self.arc() as std::sync::Arc<dyn IntoSubject>
            }

            pub fn entity(&self) -> &'static str {
                Self::ENTITY
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
