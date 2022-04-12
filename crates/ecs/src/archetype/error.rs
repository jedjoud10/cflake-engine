use crate::component::ComponentError;

// Archetype Error
pub enum ArchetypeError {
    // Specific component error
    ComponentError(ComponentError),

    // Component is not valid for the current archetype (not specified in the layout when initializing the archetype)
    Invalid(&'static str),

    // Archetype not found
    NotFound,

    // Did not link all the components needed
    IncompleteLinks,
}

impl std::fmt::Debug for ArchetypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchetypeError::ComponentError(err) => std::fmt::Debug::fmt(err, f),
            ArchetypeError::Invalid(name) => write!(f, "Component of type '{}' is invalid for the current archetype", name),
            ArchetypeError::NotFound => write!(f, "Archetype not found, you must register the archetype first"),
            ArchetypeError::IncompleteLinks => write!(f, "Missing components, check component layout or insert component calls"),
        }
    }
}

impl std::fmt::Display for ArchetypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for ArchetypeError {}

// Link modifier Error
pub enum LinkError {
    ComponentError(ComponentError),
    ComponentMissing(&'static str),
    LinkDuplication(&'static str),
    StrictLinkInvalid(&'static str),
}

impl std::fmt::Debug for LinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkError::ComponentError(err) => std::fmt::Debug::fmt(err, f),
            LinkError::LinkDuplication(name) => write!(f, "Component of type '{}' is already linked to the entity", name),
            LinkError::ComponentMissing(name) => write!(f, "Unable to remove component of type '{}' because it is missing", name),
            LinkError::StrictLinkInvalid(name) => write!(f, "Unable to add component of type '{}' because the target archetype does not accept it", name),
        }
    }
}

impl std::fmt::Display for LinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for LinkError {}
