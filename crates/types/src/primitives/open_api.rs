use super::{
    Address,
    AssetId,
    BlockHeight,
    BlockTimestamp,
    Bytes32,
    ContractId,
    HexData,
    TxId,
};

// --------------------------------------------------------------------- //
impl utoipa::ToSchema for Address {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("Address")
    }
}
impl utoipa::PartialSchema for Address {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .property(
                "id",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                        utoipa::openapi::KnownFormat::Int64,
                    ))),
            )
            .required("id")
            .property(
                "name",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::String),
            )
            .required("name")
            .property(
                "age",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                        utoipa::openapi::KnownFormat::Int32,
                    ))),
            )
            .examples([serde_json::json!({
              "name":"bob the cat","id":1
            })])
            .into()
    }
}

// --------------------------------------------------------------------- //
impl utoipa::ToSchema for BlockTimestamp {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("BlockTimestamp")
    }
}
impl utoipa::PartialSchema for BlockTimestamp {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .property(
                "id",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                        utoipa::openapi::KnownFormat::Int64,
                    ))),
            )
            .required("id")
            .property(
                "name",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::String),
            )
            .required("name")
            .property(
                "age",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                        utoipa::openapi::KnownFormat::Int32,
                    ))),
            )
            .examples([serde_json::json!({
              "name":"bob the cat","id":1
            })])
            .into()
    }
}
// --------------------------------------------------------------------- //
impl utoipa::ToSchema for BlockHeight {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("BlockHeight")
    }
}
impl utoipa::PartialSchema for BlockHeight {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .property(
                "id",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                        utoipa::openapi::KnownFormat::Int64,
                    ))),
            )
            .required("id")
            .property(
                "name",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::String),
            )
            .required("name")
            .property(
                "age",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                        utoipa::openapi::KnownFormat::Int32,
                    ))),
            )
            .examples([serde_json::json!({
              "name":"bob the cat","id":1
            })])
            .into()
    }
}
// --------------------------------------------------------------------- //
impl utoipa::ToSchema for TxId {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("TxId")
    }
}
impl utoipa::PartialSchema for TxId {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .property(
                "id",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                        utoipa::openapi::KnownFormat::Int64,
                    ))),
            )
            .required("id")
            .property(
                "name",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::String),
            )
            .required("name")
            .property(
                "age",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                        utoipa::openapi::KnownFormat::Int32,
                    ))),
            )
            .examples([serde_json::json!({
              "name":"bob the cat","id":1
            })])
            .into()
    }
}

// --------------------------------------------------------------------- //
impl utoipa::ToSchema for AssetId {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("AssetId")
    }
}
impl utoipa::PartialSchema for AssetId {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .property(
                "id",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                        utoipa::openapi::KnownFormat::Int64,
                    ))),
            )
            .required("id")
            .property(
                "name",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::String),
            )
            .required("name")
            .property(
                "age",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                        utoipa::openapi::KnownFormat::Int32,
                    ))),
            )
            .examples([serde_json::json!({
              "name":"bob the cat","id":1
            })])
            .into()
    }
}

// --------------------------------------------------------------------- //
impl utoipa::ToSchema for ContractId {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("ContractId")
    }
}
impl utoipa::PartialSchema for ContractId {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .property(
                "id",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                        utoipa::openapi::KnownFormat::Int64,
                    ))),
            )
            .required("id")
            .property(
                "name",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::String),
            )
            .required("name")
            .property(
                "age",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                        utoipa::openapi::KnownFormat::Int32,
                    ))),
            )
            .examples([serde_json::json!({
              "name":"bob the cat","id":1
            })])
            .into()
    }
}
// --------------------------------------------------------------------- //
impl utoipa::ToSchema for HexData {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("HexData")
    }
}
impl utoipa::PartialSchema for HexData {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .property(
                "id",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                        utoipa::openapi::KnownFormat::Int64,
                    ))),
            )
            .required("id")
            .property(
                "name",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::String),
            )
            .required("name")
            .property(
                "age",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                        utoipa::openapi::KnownFormat::Int32,
                    ))),
            )
            .examples([serde_json::json!({
              "name":"bob the cat","id":1
            })])
            .into()
    }
}
// --------------------------------------------------------------------- //
impl utoipa::ToSchema for Bytes32 {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("Bytes32")
    }
}
impl utoipa::PartialSchema for Bytes32 {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .property(
                "id",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                        utoipa::openapi::KnownFormat::Int64,
                    ))),
            )
            .required("id")
            .property(
                "name",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::String),
            )
            .required("name")
            .property(
                "age",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                        utoipa::openapi::KnownFormat::Int32,
                    ))),
            )
            .examples([serde_json::json!({
              "name":"bob the cat","id":1
            })])
            .into()
    }
}
