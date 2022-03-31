use crate::component::{Component, ComponentError};
use std::any::type_name;

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
            ArchetypeError::Invalid(name) => write!(
                f,
                "Component of type '{}' is invalid for the current archetype",
                name
            ),
            ArchetypeError::NotFound => write!(
                f,
                "Archetype not found, you must register the archetype first"
            ),
            ArchetypeError::IncompleteLinks => write!(
                f,
                "Missing components, check component layout or insert component calls"
            ),
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
pub enum LinkModifierError {
    // Specific component error
    ComponentError(ComponentError),

    // Component missing
    ComponentMissing(&'static str),

    // Component duplication
    LinkDuplication(&'static str),
}

impl std::fmt::Debug for LinkModifierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkModifierError::ComponentError(err) => std::fmt::Debug::fmt(err, f),
            LinkModifierError::LinkDuplication(name) => write!(
                f,
                "Component of type '{}' is already linked to the entity",
                name
            ),
            LinkModifierError::ComponentMissing(name) => write!(
                f,
                "Unable to remove component of type '{}' because it is missing",
                name
            ),
        }
    }
}

impl std::fmt::Display for LinkModifierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for LinkModifierError {}
