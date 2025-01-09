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

pub fn to_sql_where_fn(field_names: &[&Ident]) -> TokenStream {
    let field_props = field_names.iter().map(|name| {
        quote! {
            match &self.#name {
                Some(val) => Some(format!("{} = '{}'", stringify!(#name), val)),
                None => None,
            }
        }
    });

    quote! {
        fn to_sql_where(&self) -> Option<String>{
            let pattern = self.parse();
            if pattern.ends_with(".>") {
                return None;
            }

            let conditions = vec![#(#field_props),*].into_iter().filter_map(|x| x).collect::<Vec<_>>();
            Some(conditions.join(" AND "))
        }
    }
}

pub fn validate_pattern_fn(field_names: &[&Ident]) -> TokenStream {
    let field_length = field_names.len();
    quote! {
        fn validate_pattern(&self, pattern: &str) -> Result<(), fuel_streams_macros::subject::SubjectError> {
            fuel_streams_macros::subject::SubjectValidator::validate(pattern)?;
            let pattern_length = pattern.split('.').count();
            if pattern_length > #field_length {
                return Err(fuel_streams_macros::subject::SubjectError::IncompatiblePattern);
            }
            Ok(())
        }
    }
}

pub fn from_json_str_fn(field_names: &[&Ident]) -> TokenStream {
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
        fn from_json_str(json: &str) -> Result<Self, SubjectError> {
            let parsed: serde_json::Value = serde_json::from_str(json)
                .map_err(|e| SubjectError::InvalidJsonConversion(e))?;

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
