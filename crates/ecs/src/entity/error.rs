use crate::ComponentError;

pub enum EntryError {
    ComponentError(ComponentError),
    MissingComponent(&'static str),
}

impl std::fmt::Display for EntryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::fmt::Debug for EntryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryError::ComponentError(err) => std::fmt::Debug::fmt(err, f),
            EntryError::MissingComponent(name) => write!(f, "The component '{}' is not linked to the entity", name),
        }
    }
}

impl std::error::Error for EntryError {}
