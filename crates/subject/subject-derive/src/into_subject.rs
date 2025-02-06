use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, Type};

use crate::fields::FieldInfo;

pub fn parse_fn(input: &DeriveInput, field_names: &[&Ident]) -> TokenStream {
    let format_str = super::attrs::subject_attr("format", &input.attrs);
    let parse_fields = field_names.iter().map(|name| {
        quote! {
            let #name = fuel_streams_subject::subject::parse_param(&self.#name);
        }
    });

    quote! {
        fn parse(&self) -> String {
            if [#(&self.#field_names.is_none()),*].iter().all(|&x| *x) {
                return Self::QUERY_ALL.to_string();
            }
            #(#parse_fields)*
            format!(#format_str)
        }
    }
}

pub fn query_all_fn() -> TokenStream {
    quote! {
        fn query_all(&self) -> &'static str {
            Self::QUERY_ALL
        }
    }
}

pub fn to_sql_where_fn(fields: &[FieldInfo]) -> TokenStream {
    let conditions: Vec<TokenStream> = fields
        .iter()
        .map(|field| {
            let name = field.ident;
            let column_name = match &field.attributes.sql_column {
                Some(val) => val.clone(),
                None => name.to_string(),
            };

            quote! {
                if let Some(val) = &self.#name {
                    conditions.push(format!("{} = '{}'", #column_name, val));
                }
            }
        })
        .collect();

    quote! {
        fn to_sql_where(&self) -> Option<String> {
            let mut conditions = Vec::new();
            #(#conditions)*

            if let Some(extra) = Self::EXTRA_WHERE {
                if conditions.is_empty() {
                    Some(extra.to_string())
                } else {
                    Some(format!("{} AND {}", conditions.join(" AND "), extra))
                }
            } else if conditions.is_empty() {
                None
            } else {
                Some(conditions.join(" AND "))
            }
        }
    }
}

pub fn to_sql_select_fn(fields: &[FieldInfo]) -> TokenStream {
    let column_names: Vec<TokenStream> = fields
        .iter()
        .map(|field| {
            let name = field.ident;
            let column_name = match &field.attributes.sql_column {
                Some(val) => val.clone(),
                None => name.to_string(),
            };

            quote! {
                if self.#name.is_some() {
                    columns.push(#column_name.to_string());
                }
            }
        })
        .collect();

    quote! {
        fn to_sql_select(&self) -> Option<String> {
            let mut columns = Vec::new();
            #(#column_names)*
            if columns.is_empty() {
                None
            } else {
                Some(columns.join(", "))
            }
        }
    }
}

pub fn id_fn() -> TokenStream {
    quote! {
        fn id(&self) -> &'static str {
            Self::ID
        }
    }
}

pub fn to_payload_fn(input: &DeriveInput) -> TokenStream {
    let name = &input.ident.to_string();
    let crate_path = quote!(fuel_streams_subject::subject);
    quote! {
        fn to_payload(&self) -> #crate_path::SubjectPayload {
            let params = #crate_path::serde_json::to_value(self).expect(&format!("Failed to serialize {}", stringify!(#name)));
            #crate_path::SubjectPayload {
                params: params.to_owned(),
                subject: self.id().to_string(),
            }
        }
    }
}

pub fn schema_fn(
    input: &DeriveInput,
    field_infos: &[FieldInfo],
    field_names: &[&Ident],
    field_types: &[&Type],
) -> TokenStream {
    let id = super::attrs::subject_attr("id", &input.attrs);
    let entity = super::attrs::subject_attr("entity", &input.attrs);
    let format = super::attrs::subject_attr("format", &input.attrs);
    let query_all = super::attrs::subject_attr("query_all", &input.attrs);
    let struct_name = &input.ident.to_string();

    let field_entries =
        field_names
            .iter()
            .zip(field_types.iter())
            .map(|(name, ty)| {
                let name_str = name.to_string();
                let type_str = quote::quote!(#ty)
                    .to_string()
                    .replace("Option < ", "")
                    .replace(" >", "");

                let field_info = field_infos.iter().find(|i| i.ident == *name);
                let description =
                    field_info.and_then(|i| i.attributes.description.clone());
                let description_quote = match description {
                    Some(desc) => quote!(Some(#desc.to_string())),
                    None => quote!(None),
                };

                quote! {
                    fields.insert(
                        #name_str.to_string(),
                        fuel_streams_subject::subject::FieldSchema {
                            type_name: #type_str.to_string(),
                            description: #description_quote,
                        }
                    );
                }
            });

    quote! {
        fn schema(&self) -> fuel_streams_subject::subject::Schema {
            let mut fields = fuel_streams_subject::subject::IndexMap::new();
            #(#field_entries)*

            fuel_streams_subject::subject::Schema {
                id: #id.to_string(),
                entity: #entity.to_string(),
                subject: #struct_name.to_string(),
                format: #format.to_string(),
                query_all: #query_all.to_string(),
                fields,
                variants: None,
            }
        }
    }
}
