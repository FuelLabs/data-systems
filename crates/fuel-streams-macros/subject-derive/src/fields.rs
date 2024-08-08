use proc_macro2::TokenStream;
use syn::{
    punctuated::Punctuated,
    token::Comma,
    Data,
    DeriveInput,
    Error,
    Field,
    Fields,
    Ident,
    Path,
    PathSegment,
    Type,
    TypePath,
};

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = ty
    {
        if let Some(PathSegment { ident, .. }) = segments.first() {
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

    // Check if all fields are Option<>
    for field in fields.iter() {
        if !is_option_type(&field.ty) {
            return Err(Error::new_spanned(
                field,
                "All fields in a Subject struct must be Option<>",
            )
            .to_compile_error());
        }
    }

    Ok(fields)
}

pub fn names_from_fields(fields: &Punctuated<Field, Comma>) -> Vec<&Ident> {
    fields.iter().filter_map(|f| f.ident.as_ref()).collect()
}

pub fn types_from_fields(fields: &Punctuated<Field, Comma>) -> Vec<&Type> {
    fields.iter().map(|f| &f.ty).collect()
}
