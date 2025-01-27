use proc_macro2::{TokenStream, TokenTree};
use syn::{
    punctuated::Punctuated,
    token::Comma,
    Data,
    DeriveInput,
    Error,
    Field,
    Fields,
    Ident,
    Meta,
    Type,
    TypePath,
};

#[derive(Debug, Default)]
pub struct FieldAttributes {
    pub sql_column: Option<String>,
}

impl FieldAttributes {
    fn from_field(field: &Field) -> Self {
        field
            .attrs
            .iter()
            .filter(|attr| attr.meta.path().is_ident("subject"))
            .fold(FieldAttributes::default(), |mut attrs, attr| {
                Self::parse_attr_values(attr, &mut attrs);
                attrs
            })
    }

    fn parse_token_stream(&mut self, tokens: &TokenStream) {
        let mut tokens = tokens.clone().into_iter();
        while let Some(token) = tokens.next() {
            let parsed = self.parse_name_value_pair(&mut tokens, token);
            if let Some((name, value)) = parsed {
                self.set_attribute(name, value);
            }
        }
    }

    fn parse_name_value_pair(
        &self,
        tokens: &mut impl Iterator<Item = TokenTree>,
        name_token: TokenTree,
    ) -> Option<(String, String)> {
        if let TokenTree::Ident(name) = name_token {
            // Skip the equals sign
            if let Some(TokenTree::Punct(punct)) = tokens.next() {
                if punct.as_char() == '=' {
                    // Get the string literal
                    if let Some(TokenTree::Literal(lit)) = tokens.next() {
                        let lit_str = lit.to_string();
                        // Remove the quotes
                        let value = lit_str[1..lit_str.len() - 1].to_string();
                        return Some((name.to_string(), value));
                    }
                }
            }
        }
        None
    }

    fn set_attribute(&mut self, name: String, value: String) {
        if name.as_str() == "sql_column" {
            self.sql_column = Some(value)
        }
    }

    fn parse_attr_values(attr: &syn::Attribute, attrs: &mut FieldAttributes) {
        if let Ok(meta) = attr.parse_args::<Meta>() {
            match meta {
                Meta::List(list) => {
                    attrs.parse_token_stream(&list.tokens);
                }
                Meta::NameValue(name_value) => {
                    if name_value.path.is_ident("sql_column") {
                        if let syn::Expr::Lit(expr_lit) = name_value.value {
                            if let syn::Lit::Str(lit_str) = expr_lit.lit {
                                attrs.sql_column = Some(lit_str.value());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug)]
pub struct FieldInfo<'a> {
    pub ident: &'a Ident,
    pub attributes: FieldAttributes,
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(TypePath {
        path: syn::Path { segments, .. },
        ..
    }) = ty
    {
        if let Some(syn::PathSegment { ident, .. }) = segments.first() {
            return ident == "Option";
        }
    }
    false
}

pub fn from_input(
    input: &DeriveInput,
) -> Result<&Punctuated<Field, Comma>, TokenStream> {
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => &fields_named.named,
            _ => return Err(Error::new_spanned(
                input,
                "Subject derive macro only supports structs with named fields",
            )
            .to_compile_error()),
        },
        _ => {
            return Err(Error::new_spanned(
                input,
                "Subject derive macro only supports structs",
            )
            .to_compile_error())
        }
    };

    validate_option_fields(fields)?;
    Ok(fields)
}

fn validate_option_fields(
    fields: &Punctuated<Field, Comma>,
) -> Result<(), TokenStream> {
    for field in fields.iter() {
        if !is_option_type(&field.ty) {
            return Err(Error::new_spanned(
                field,
                "All fields in a Subject struct must be Option<>",
            )
            .to_compile_error());
        }
    }
    Ok(())
}

pub fn names_from_fields(fields: &Punctuated<Field, Comma>) -> Vec<FieldInfo> {
    fields
        .iter()
        .filter_map(|f| {
            let attributes = FieldAttributes::from_field(f);
            f.ident
                .as_ref()
                .map(|ident| FieldInfo { ident, attributes })
        })
        .collect()
}

pub fn types_from_fields(fields: &Punctuated<Field, Comma>) -> Vec<&Type> {
    fields.iter().map(|f| &f.ty).collect()
}
