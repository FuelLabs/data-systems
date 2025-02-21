use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Type};

use crate::{attrs::SubjectAttrs, fields::FieldInfo};

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
    field_infos: &'a [FieldInfo<'a>],
    attrs: &'a [syn::Attribute],
) -> TokenStream {
    let with_methods = create_with_methods(field_names, field_types);
    let get_methods = create_get_methods(field_names, field_types);
    let query_all = crate::attrs::subject_attr("query_all", attrs);
    let id = crate::attrs::subject_attr("id", attrs);
    let entity = crate::attrs::subject_attr("entity", attrs);
    let crate_path = quote!(fuel_streams_subject::subject);

    let custom_where = if let Some(extra) =
        SubjectAttrs::from_attributes(attrs).get("custom_where")
    {
        quote! { Some(#extra) }
    } else {
        quote! { None }
    };

    let parse_fields = field_infos.iter().map(|field_info| {
        let name = field_info.ident;
        let name_str = name.to_string();
        let value_getter = if let Some(alias) = &field_info.attributes.alias {
            quote! {
                obj.get(#name_str).or_else(|| obj.get(#alias))
            }
        } else {
            quote! {
                obj.get(#name_str)
            }
        };

        quote! {
            let #name = #value_getter
                .and_then(|value| {
                    if value.is_null() {
                        None
                    } else {
                        Some(value.to_string().trim_matches('"').to_string())
                    }
                });
        }
    });

    let validate_fields = {
        let field_name_strings: Vec<String> =
            field_names.iter().map(|f| f.to_string()).collect();
        let aliases: Vec<String> = field_infos
            .iter()
            .filter_map(|f| f.attributes.alias.as_ref())
            .map(|a| a.to_string())
            .collect();

        quote! {
            fn validate_obj(obj: &serde_json::Map<String, serde_json::Value>) -> Result<(), #crate_path::SubjectPayloadError> {
                let valid_keys: std::collections::HashSet<String> = obj.keys()
                    .map(|k| k.to_string())
                    .collect();

                let valid_field_names = [#(#field_name_strings),*];
                let valid_aliases = [#(#aliases),*];
                for key in valid_keys.iter() {
                    let is_valid = valid_field_names.iter()
                        .chain(valid_aliases.iter())
                        .any(|valid| valid == key);

                    if !is_valid {
                        return Err(#crate_path::SubjectPayloadError::InvalidParams(format!(
                            "Unknown field in params: {}",
                            key
                        )));
                    }
                }
                Ok(())
            }

            validate_obj(obj)?;
        }
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

        impl TryFrom<#crate_path::SubjectPayload> for #name {
            type Error = #crate_path::SubjectPayloadError;
            fn try_from(payload: #crate_path::SubjectPayload) -> Result<Self, Self::Error> {
                let obj = match payload.params.as_object() {
                    Some(obj) => obj,
                    None => return Err(#crate_path::SubjectPayloadError::ExpectedJsonObject),
                };

                #validate_fields
                #(#parse_fields)*

                let payload = Self::build(#(#field_names.and_then(|v| v.parse().ok()),)*);
                Ok(payload)
            }
        }
    }
}
