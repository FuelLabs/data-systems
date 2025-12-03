#[macro_export]
macro_rules! impl_utoipa_for_byte_type_detailed {
    // Fixed-size type implementation with custom description
    ($wrapper_type:ident, $byte_size:expr, $description:expr) => {
        impl utoipa::ToSchema for $wrapper_type {
            fn name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(stringify!($wrapper_type))
            }
        }

        impl utoipa::PartialSchema for $wrapper_type {
            fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
                utoipa::openapi::schema::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::String)
                    .format(Some(utoipa::openapi::schema::SchemaFormat::Custom(
                        "hex".to_string(),
                    )))
                    .description(Some($description))
                    .examples([Some(serde_json::json!(format!(
                        "0x{}",
                        "0".repeat($byte_size * 2)
                    )))])
                    .pattern(Some(format!("^0x[0-9a-fA-F]{{{}}}$", $byte_size * 2)))
                    .min_length(Some(2 + $byte_size * 2))
                    .max_length(Some(2 + $byte_size * 2))
                    .build()
                    .into()
            }
        }
    };

    // Variable-size type implementation with custom description
    ($wrapper_type:ident, $description:expr) => {
        impl utoipa::ToSchema for $wrapper_type {
            fn name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(stringify!($wrapper_type))
            }
        }

        impl utoipa::PartialSchema for $wrapper_type {
            fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
                utoipa::openapi::schema::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::String)
                    .format(Some(utoipa::openapi::schema::SchemaFormat::Custom(
                        "hex".to_string(),
                    )))
                    .description(Some($description))
                    .examples([Some(serde_json::json!("0x00"))])
                    .pattern(Some("^0x[0-9a-fA-F]+$"))
                    .build()
                    .into()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_utoipa_for_integer_wrapper {
    // Basic implementation with just type name and description
    ($wrapper_type:ident, $description:expr) => {
        impl utoipa::ToSchema for $wrapper_type {
            fn name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(stringify!($wrapper_type))
            }
        }

        impl utoipa::PartialSchema for $wrapper_type {
            fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
                utoipa::openapi::schema::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::schema::SchemaFormat::Int64))
                    .description(Some($description))
                    .examples([Some(serde_json::json!(0))])
                    .build()
                    .into()
            }
        }
    };

    // Implementation with min/max values and description
    ($wrapper_type:ident, $description:expr, $min:expr, $max:expr) => {
        impl utoipa::ToSchema for $wrapper_type {
            fn name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(stringify!($wrapper_type))
            }
        }

        impl utoipa::PartialSchema for $wrapper_type {
            fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
                utoipa::openapi::schema::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                        utoipa::openapi::KnownFormat::Int64,
                    )))
                    .description(Some($description))
                    .examples([Some(serde_json::json!(0))])
                    .minimum(Some(utoipa::Number::UInt($min)))
                    .maximum(Some(utoipa::Number::UInt($max)))
                    .build()
                    .into()
            }
        }
    };

    // Implementation with custom format (u32, i32, etc.)
    ($wrapper_type:ident, $description:expr, $format:expr) => {
        impl utoipa::ToSchema for $wrapper_type {
            fn name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(stringify!($wrapper_type))
            }
        }

        impl utoipa::PartialSchema for $wrapper_type {
            fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
                utoipa::openapi::schema::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::schema::SchemaFormat::Custom(
                        $format.to_string(),
                    )))
                    .description(Some($description))
                    .examples([Some(serde_json::json!(0))])
                    .build()
                    .into()
            }
        }
    };

    // Implementation with custom format and min/max values
    (
        $wrapper_type:ident,
        $description:expr,
        $format:expr,
        $min:expr,
        $max:expr
    ) => {
        impl utoipa::ToSchema for $wrapper_type {
            fn name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(stringify!($wrapper_type))
            }
        }

        impl utoipa::PartialSchema for $wrapper_type {
            fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
                utoipa::openapi::schema::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::schema::SchemaFormat::Custom(
                        $format.to_string(),
                    )))
                    .description(Some($description))
                    .examples([Some(serde_json::json!(0))])
                    .minimum(Some(serde_json::json!($min)))
                    .maximum(Some(serde_json::json!($max)))
                    .build()
                    .into()
            }
        }
    };
}
