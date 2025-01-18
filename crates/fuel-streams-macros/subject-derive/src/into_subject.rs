use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, Type};

use crate::fields::FieldInfo;

pub fn parse_fn(input: &DeriveInput, field_names: &[&Ident]) -> TokenStream {
    let format_str = super::attrs::subject_attr("format", &input.attrs);
    let parse_fields = field_names.iter().map(|name| {
        quote! {
            let #name = fuel_streams_macros::subject::parse_param(&self.#name);
        }
    });

    quote! {
        fn parse(&self) -> String {
            if [#(&self.#field_names.is_none()),*].iter().all(|&x| *x) {
                return Self::WILDCARD.to_string();
            }
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
        fn to_sql_where(&self) -> String {
            let mut conditions = Vec::new();
            #(#conditions)*
            if conditions.is_empty() {
                "TRUE".to_string()
            } else {
                conditions.join(" AND ")
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
        fn to_sql_select(&self) -> String {
            let mut columns = Vec::new();
            #(#column_names)*
            if columns.is_empty() {
                "*".to_string()
            } else {
                columns.join(", ")
            }
        }
    }
}

pub fn from_json_fn(field_names: &[&Ident]) -> TokenStream {
    let parse_fields = field_names.iter().map(|name| {
        let name_str = name.to_string();
        quote! {
            let #name = if let Some(value) = obj.get(#name_str) {
                if value.is_null() {
                    None
                } else {
                    let str_val = value.to_string().trim_matches('"').to_string();
                    Some(str_val)
                }
            } else {
                None
            };
        }
    });

    quote! {
        fn from_json(json: &str) -> Result<Self, SubjectError> {
            let parsed: fuel_streams_macros::subject::serde_json::Value =
                fuel_streams_macros::subject::serde_json::from_str(json)
                    .map_err(|e| SubjectError::InvalidJsonConversion(e.to_string()))?;

            let obj = match parsed.as_object() {
                Some(obj) => obj,
                None => return Err(SubjectError::ExpectedJsonObject),
            };

            #(#parse_fields)*

            Ok(Self::build(
                #(#field_names.and_then(|v| v.parse().ok()),)*
            ))
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

pub fn to_json_fn() -> TokenStream {
    quote! {
        fn to_json(&self) -> String {
            fuel_streams_macros::subject::serde_json::to_string(self).unwrap()
        }
    }
}

pub fn schema_fn(
    input: &DeriveInput,
    field_names: &[&Ident],
    field_types: &[&Type],
) -> TokenStream {
    let id = super::attrs::subject_attr("id", &input.attrs);
    let entity = super::attrs::subject_attr("entity", &input.attrs);
    let format = super::attrs::subject_attr("format", &input.attrs);
    let wildcard = super::attrs::subject_attr("wildcard", &input.attrs);
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

                quote! {
                    fields.insert(
                        #name_str.to_string(),
                        fuel_streams_macros::subject::FieldSchema {
                            type_name: #type_str.to_string(),
                        }
                    );
                }
            });

    quote! {
        fn schema(&self) -> fuel_streams_macros::subject::Schema {
            let mut fields = std::collections::HashMap::new();
            #(#field_entries)*

            fuel_streams_macros::subject::Schema {
                id: #id.to_string(),
                entity: #entity.to_string(),
                subject: #struct_name.to_string(),
                format: #format.to_string(),
                wildcard: #wildcard.to_string(),
                fields,
                variants: None,
            }
        }
    }
}
