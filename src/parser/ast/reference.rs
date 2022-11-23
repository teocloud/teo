#[derive(Debug, Clone, Copy)]
pub(crate) struct IdReference {
    pub(crate) source_id: usize,
    pub(crate) top_id: usize,
}

#[derive(Debug, Clone)]
pub(crate) enum Reference {
    ModelReference(IdReference),
    ConstantReference(IdReference),
}

impl Reference {

    pub(crate) fn as_model_ref(&self) -> Option<IdReference> {
        match self {
            Reference::ModelReference(r) => Some(r.clone()),
            Reference::ConstantReference(_) => None,
        }
    }

    pub(crate) fn is_model_ref(&self) -> bool {
        self.as_model_ref().is_some()
    }

    pub(crate) fn as_constant_ref(&self) -> Option<IdReference> {
        match self {
            Reference::ModelReference(_) => None,
            Reference::ConstantReference(c) => Some(c.clone()),
        }
    }

    pub(crate) fn is_constant_ref(&self) -> bool {
        self.as_constant_ref().is_some()
    }
}