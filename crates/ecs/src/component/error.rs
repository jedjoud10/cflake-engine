use std::any::type_name;

use super::Component;

// Specific component errors
pub enum ComponentError {
    // Component is not registered
    NotRegistered(&'static str),
}

impl std::fmt::Display for ComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::fmt::Debug for ComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentError::NotRegistered(name) => {
                write!(f, "Component of type '{}' is unregistered", name)
            }
        }
    }
}

impl std::error::Error for ComponentError {}

// Helper functions
pub(super) fn component_err<T: Component>(err: ComponentError) -> QueryError {
    QueryError::ComponentError(err)
}
pub(super) fn unlinked_err<T: Component>() -> QueryError {
    QueryError::Unlinked(type_name::<T>())
}
// Query Error
pub enum QueryError {
    // Specific Component error
    ComponentError(ComponentError),

    // Component is not in the current query
    Unlinked(&'static str),
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::fmt::Debug for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::ComponentError(err) => std::fmt::Debug::fmt(err, f),
            QueryError::Unlinked(name) => write!(f, "Component of type '{}' is not available in the current query.", name),
        }
    }
}

impl std::error::Error for QueryError {}
