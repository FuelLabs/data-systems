mod attrs;
mod fields;
mod parse_fn;
mod subject;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// The `Subject` derive macro allows you to easily implement the `Subject` trait for your structs.
/// It generates methods for parsing, building, and creating wildcards for your subject.
///
/// # Example
///
/// ```
/// use subject_derive::Subject;
///
/// #[derive(Subject, Debug, Clone, Default)]
/// #[subject_wildcard = "test.>"]
/// #[subject_format = "test.{field1}.{field2}"]
/// struct TestSubject {
///     field1: Option<String>,
///     field2: Option<u32>,
/// }
///
/// // Create a new TestSubject
/// let subject = TestSubject {
///     field1: Some("foo".to_string()),
///     field2: Some(55),
/// };
///
/// // Parse the subject
/// assert_eq!(subject.parse(), "test.foo.55");
///
/// // Create a wildcard
/// assert_eq!(TestSubject::wildcard(None, Some(10)), "test.*.10");
///
/// // Build a subject
/// let built_subject = TestSubject::build(Some("bar".into()), Some(42));
/// assert_eq!(built_subject.parse(), "test.bar.42");
///
/// // Use the builder pattern
/// let builder_subject = TestSubject::new()
///     .with_field1(Some("baz".into()))
///     .with_field2(Some(99));
/// assert_eq!(builder_subject.parse(), "test.baz.99");
///
/// // Convert to string
/// assert_eq!(builder_subject.to_string(), "test.baz.99");
/// ```
#[proc_macro_derive(Subject, attributes(subject_wildcard, subject_format))]
pub fn subject_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let fields = fields::from_input(&input).unwrap();
    let field_names = fields::names_from_fields(fields);
    let field_types = fields::types_from_fields(fields);
    let wildcard = attrs::subject_attr("wildcard", &input.attrs);
    let parse_fn = parse_fn::create(&input, &field_names);
    let subject_expanded = subject::expanded(name, &field_names, &field_types);

    quote! {
        #subject_expanded

        impl IntoSubject for #name {
            const WILDCARD: &'static str = #wildcard;
            #parse_fn

            fn new() -> Self {
                Self {
                    #(#field_names: None,)*
                }
            }
        }
    }
    .into()
}
