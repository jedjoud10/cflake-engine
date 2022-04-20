// Link modifier Error
pub enum LinkError {
    ComponentMissing(&'static str),
    LinkDuplication(&'static str),
}

impl std::fmt::Debug for LinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkError::LinkDuplication(name) => write!(f, "Component of type '{}' is already linked to the entity", name),
            LinkError::ComponentMissing(name) => write!(f, "Unable to remove component of type '{}' because it is missing", name),
        }
    }
}

impl std::fmt::Display for LinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for LinkError {}
