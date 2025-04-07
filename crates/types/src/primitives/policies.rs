use serde::{Deserialize, Serialize};

use crate::{WrappedU32, WrappedU64};

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Default, Hash,
)]
pub struct Policies {
    pub tip: Option<WrappedU64>,
    pub maturity: Option<WrappedU32>,
    pub witness_limit: Option<WrappedU64>,
    pub max_fee: Option<WrappedU64>,
}

impl Policies {
    pub fn random() -> Self {
        Self {
            tip: Some(WrappedU64::random()),
            maturity: Some(WrappedU32::random()),
            witness_limit: Some(WrappedU64::random()),
            max_fee: Some(WrappedU64::random()),
        }
    }

    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
}

impl TryFrom<String> for Policies {
    type Error = serde_json::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let policies: Policies = serde_json::from_str(&value)?;
        Ok(policies)
    }
}

impl From<fuel_tx::policies::Policies> for Policies {
    fn from(policies: fuel_tx::policies::Policies) -> Self {
        Self {
            tip: policies
                .get(fuel_tx::policies::PolicyType::Tip)
                .map(WrappedU64::from),
            maturity: policies
                .get(fuel_tx::policies::PolicyType::Maturity)
                .map(|v| WrappedU32::from(v as u32)),
            witness_limit: policies
                .get(fuel_tx::policies::PolicyType::WitnessLimit)
                .map(WrappedU64::from),
            max_fee: policies
                .get(fuel_tx::policies::PolicyType::MaxFee)
                .map(WrappedU64::from),
        }
    }
}

impl utoipa::ToSchema for Policies {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("Policies")
    }
}

impl utoipa::PartialSchema for Policies {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::schema::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::Type::Array)
            .title(Some("Policies"))
            .description(Some("Array of u64 policy values used by the VM"))
            .property(
                "values",
                utoipa::openapi::schema::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(
                        utoipa::openapi::schema::SchemaFormat::KnownFormat(
                            utoipa::openapi::KnownFormat::Int64,
                        ),
                    ))
                    .build(),
            )
            .examples([Some(serde_json::json!([0, 0, 0, 0, 0]))])
            .build()
            .into()
    }
}
