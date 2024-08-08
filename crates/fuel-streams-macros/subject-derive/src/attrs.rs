use syn::{Attribute, Expr, Lit};

fn find_attr<'a>(name: &'a str, attrs: &'a [Attribute]) -> &'a Attribute {
    attrs
        .iter()
        .find(|attr| {
            let segments = attr.path().clone().segments;
            let first = segments.first().unwrap();
            first.ident.to_string().contains(name)
        })
        .unwrap_or_else(|| panic!("#[subject_{}] attribute not defined", name))
}

fn find_literal_str(name: &str, attr: &Attribute) -> String {
    let meta = attr.meta.require_name_value().unwrap();
    if let Expr::Lit(arg) = meta.value.clone() {
        match arg.lit {
            Lit::Str(lit) => Some(lit.value()),
            _ => None,
        }
    } else {
        None
    }
    .unwrap_or_else(|| panic!("#[subject_{}] is not a valid string", name))
}

pub fn subject_attr(name: &str, attrs: &[Attribute]) -> String {
    let attr = find_attr(name, attrs);
    find_literal_str(name, attr)
}
