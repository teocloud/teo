use serde_json::{Value as JsonValue};
use crate::core::field::Optionality;
use crate::core::object::Object;

pub mod builder;

#[derive(Debug, Clone)]
pub(crate) struct Relation {
    pub(crate) name: String,
    pub(crate) localized_name: String,
    pub(crate) description: String,
    pub(crate) optionality: Optionality,
    pub(crate) model: String,
    pub(crate) through: Option<String>,
    pub(crate) is_vec: bool,
    pub(crate) fields: Vec<String>,
    pub(crate) references: Vec<String>,
}

impl Relation {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn localized_name(&self) -> &str {
        &self.localized_name
    }

    pub(crate) fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug)]
pub(crate) enum RelationManipulation {
    Create(JsonValue),
    CreateOrConnect(JsonValue),
    Connect(JsonValue),
    Set(JsonValue),
    Update(JsonValue),
    Upsert(JsonValue),
    Disconnect(JsonValue),
    Delete(JsonValue),
}

#[derive(Debug)]
pub(crate) enum RelationConnection {
    Link(Object),
    Unlink(Object),
    UnlinkAndDelete(Object),
}
