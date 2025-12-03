use std::fmt;

use fuel_core_types::fuel_tx::UpgradePurpose as FuelCoreUpgradePurpose;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct UpgradePurpose(pub FuelCoreUpgradePurpose);

impl From<FuelCoreUpgradePurpose> for UpgradePurpose {
    fn from(purpose: FuelCoreUpgradePurpose) -> Self {
        UpgradePurpose(purpose)
    }
}

impl From<UpgradePurpose> for FuelCoreUpgradePurpose {
    fn from(wrapper: UpgradePurpose) -> Self {
        wrapper.0
    }
}

impl utoipa::ToSchema for UpgradePurpose {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("UpgradePurpose")
    }
}

impl utoipa::PartialSchema for UpgradePurpose {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        // Create Object builders first
        let consensus_params_obj = utoipa::openapi::schema::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::Type::Object)
            .title(Some("ConsensusParameters"))
            // ... other properties
            .build();

        let state_transition_obj = utoipa::openapi::schema::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::Type::Object)
            .title(Some("StateTransition"))
            // ... other properties
            .build();

        // Convert Objects to Schemas
        let consensus_params =
            utoipa::openapi::schema::Schema::Object(consensus_params_obj);
        let state_transition =
            utoipa::openapi::schema::Schema::Object(state_transition_obj);

        // Create a oneOf schema with both variants
        let mut one_of = utoipa::openapi::schema::OneOf::new();

        // Now we can add Schemas to the items
        one_of
            .items
            .push(utoipa::openapi::RefOr::T(consensus_params));
        one_of
            .items
            .push(utoipa::openapi::RefOr::T(state_transition));

        // Create the oneOf schema and return it
        let schema = utoipa::openapi::schema::Schema::OneOf(one_of);

        // Return the Schema
        utoipa::openapi::RefOr::T(schema)
    }
}

impl fmt::Display for UpgradePurpose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            FuelCoreUpgradePurpose::ConsensusParameters { checksum, .. } => {
                let encoded = hex::encode(checksum);
                write!(f, "0x{encoded}")
            }
            FuelCoreUpgradePurpose::StateTransition { root } => {
                let encoded = hex::encode(root);
                write!(f, "0x{encoded}")
            }
        }
    }
}
