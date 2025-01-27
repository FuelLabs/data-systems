mod attrs;
mod fields;
mod into_subject;
mod subject;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Subject, attributes(subject))]
pub fn subject_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = fields::from_input(&input).unwrap();
    let field_infos = fields::names_from_fields(fields);
    let field_names: Vec<_> = field_infos.iter().map(|f| f.ident).collect();
    let field_types = fields::types_from_fields(fields);

    let id_fn = into_subject::id_fn();
    let query_all_fn = into_subject::query_all_fn();
    let parse_fn = into_subject::parse_fn(&input, &field_names);
    let to_sql_where_fn = into_subject::to_sql_where_fn(&field_infos);
    let to_sql_select_fn = into_subject::to_sql_select_fn(&field_infos);
    let from_json_fn = into_subject::from_json_fn(&field_names);
    let to_json_fn = into_subject::to_json_fn();
    let schema_fn = into_subject::schema_fn(&input, &field_names, &field_types);
    let subject_expanded =
        subject::expanded(name, &field_names, &field_types, &input.attrs);

    quote! {
        #subject_expanded

        impl fuel_streams_macros::subject::SubjectBuildable for #name {
            fn new() -> Self {
                Self {
                    #(#field_names: None,)*
                }
            }
        }

        impl IntoSubject for #name {
            #id_fn
            #parse_fn
            #query_all_fn
            #to_sql_where_fn
            #to_sql_select_fn
            #schema_fn
        }

        impl FromJsonString for #name {
            #from_json_fn
            #to_json_fn
        }
    }
    .into()
}
