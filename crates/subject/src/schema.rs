pub use std::fmt::Debug;

pub use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
pub use serde_json;

pub use crate::payload::*;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldSchema {
    #[serde(rename = "type")]
    pub type_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Schema {
    pub id: String,
    pub entity: String,
    pub subject: String,
    pub format: String,
    #[serde(rename = "wildcard")]
    pub query_all: String,
    pub fields: IndexMap<String, FieldSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<IndexMap<String, Schema>>,
}
impl Schema {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    pub fn set_variant(&mut self, name: String, variant: Schema) -> &mut Self {
        if self.variants.is_none() {
            self.variants = Some(IndexMap::new());
        }
        self.variants.as_mut().unwrap().insert(name, variant);
        self
    }
}
