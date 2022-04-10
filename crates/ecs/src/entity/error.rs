use crate::ComponentError;

pub enum EntityEntryError {
    ComponentError(ComponentError),
    MissingComponent(&'static str),
}

impl std::fmt::Display for EntityEntryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::fmt::Debug for EntityEntryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityEntryError::ComponentError(err) => std::fmt::Debug::fmt(err, f),
            EntityEntryError::MissingComponent(name) => write!(f, "The component '{}' is not linked to the entity", name),
        }
    }
}

impl std::error::Error for EntityEntryError {}
