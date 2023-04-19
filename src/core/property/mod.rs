use inflector::Inflector;
use crate::app::ctx::AppCtx;
use crate::core::database::r#type::DatabaseType;
use crate::core::field::optionality::Optionality;
use crate::core::field::r#type::{FieldType, FieldTypeOwner};
use crate::core::pipeline::Pipeline;
use crate::core::result::Result;

#[derive(Clone)]
pub struct Property {
    pub(crate) name: &'static str,
    pub(crate) localized_name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) optionality: Optionality,
    pub(crate) field_type: Option<FieldType>,
    pub(crate) database_type: Option<DatabaseType>,
    pub(crate) dependencies: Vec<&'static str>,
    pub(crate) setter: Option<Pipeline>,
    pub(crate) getter: Option<Pipeline>,
    pub(crate) input_omissible: bool,
    pub(crate) cached: bool,
}

impl Property {

    pub(crate) fn new(name: &'static str) -> Self {
        Self {
            name,
            localized_name: None,
            description: None,
            optionality: Optionality::Optional,
            field_type: None,
            database_type: None,
            dependencies: vec![],
            setter: None,
            getter: None,
            cached: false,
            input_omissible: false,
        }
    }

    pub(crate) fn localized_name(&self) -> String {
        if let Some(ln) = &self.localized_name {
            return ln.clone()
        } else {
            self.name.to_title_case()
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn database_type(&self) -> &DatabaseType {
        self.database_type.as_ref().unwrap()
    }

    pub(crate) fn is_required(&self) -> bool {
        self.optionality.is_required()
    }

    pub(crate) fn finalize(&mut self) -> Result<()> {
        self.database_type = Some(AppCtx::get()?.connector_conf()?.default_database_type(self.field_type()));
        Ok(())
    }

    pub(crate) fn set_required(&mut self) {
        self.optionality = Optionality::Required;
    }

    pub(crate) fn set_optional(&mut self) {
        self.optionality = Optionality::Optional;
    }

    pub fn has_setter(&self) -> bool {
        self.setter.is_some()
    }

    pub fn has_getter(&self) -> bool {
        self.getter.is_some()
    }
}

impl FieldTypeOwner for Property {
    fn field_type(&self) -> &FieldType {
        self.field_type.as_ref().unwrap()
    }

    fn is_optional(&self) -> bool {
        self.optionality.is_optional()
    }

    fn set_field_type(&mut self, field_type: FieldType) {
        self.field_type = Some(field_type);
    }
}
