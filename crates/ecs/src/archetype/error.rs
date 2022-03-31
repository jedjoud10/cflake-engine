use std::any::type_name;

use crate::component::{Component, ComponentError};

// Helper functions
pub(super) fn component_err<T: Component>(err: ComponentError) -> ArchetypeError {
    ArchetypeError::ComponentError(err)
}
pub(super) fn invalid_er<T: Component>() -> ArchetypeError {
    ArchetypeError::Invalid(type_name::<T>())
}
pub(super) fn duplicate_err<T: Component>() -> ArchetypeError {
    ArchetypeError::LinkDuplication(type_name::<T>())
}

// Archetype Error
pub enum ArchetypeError {
    // Specific component error
    ComponentError(ComponentError),

    // Component is not valid for the current archetype (not specified in the layout when initializing the archetype)
    Invalid(&'static str),

    // Component is already linked
    LinkDuplication(&'static str),

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
            ArchetypeError::LinkDuplication(name) => write!(f, "Component of type '{}' is already linked to the entity", name),
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
