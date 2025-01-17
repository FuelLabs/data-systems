use proc_macro2::{TokenStream, TokenTree};
use syn::{Attribute, Meta};

#[derive(Default)]
pub struct SubjectAttrs {
    id: Option<String>,
    wildcard: Option<String>,
    format: Option<String>,
}

impl SubjectAttrs {
    pub fn from_attributes(attrs: &[Attribute]) -> Self {
        let mut subject_attrs = SubjectAttrs::default();

        for attr in attrs {
            if !attr.path().is_ident("subject") {
                continue;
            }

            if let Meta::List(list) = &attr.meta {
                subject_attrs.parse_token_stream(&list.tokens);
            }
        }

        subject_attrs
    }

    fn parse_token_stream(&mut self, tokens: &TokenStream) {
        let mut tokens = tokens.clone().into_iter();

        while let Some(token) = tokens.next() {
            if let Some((name, value)) =
                self.parse_name_value_pair(&mut tokens, token)
            {
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
        match name.as_str() {
            "id" => self.id = Some(value),
            "wildcard" => self.wildcard = Some(value),
            "format" => self.format = Some(value),
            _ => {}
        }
    }

    pub fn get(&self, name: &str) -> Option<&String> {
        match name {
            "id" => self.id.as_ref(),
            "wildcard" => self.wildcard.as_ref(),
            "format" => self.format.as_ref(),
            _ => None,
        }
    }
}

pub fn subject_attr(name: &str, attrs: &[Attribute]) -> String {
    let subject_attrs = SubjectAttrs::from_attributes(attrs);
    subject_attrs.get(name).cloned().unwrap_or_else(|| {
        panic!("No {} parameter found in #[subject] attribute", name)
    })
}
