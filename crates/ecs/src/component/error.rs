use crate::Mask;

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
                write!(
                    f,
                    "Component of type '{}' is unregistered. You must manually register the type using registry::register()",
                    name
                )
            }
        }
    }
}

impl std::error::Error for ComponentError {}

// Query Builder Error
pub enum QueryError {
    // Specific Component error
    ComponentError(ComponentError),

    // Error that occurs when we try to get a query of a component that was not specified in the entry masks
    Unlinked(&'static str),

    // Error that occurs whenever we are trying to read from a query that is currently being mutably borrowed
    MutablyBorrowed(&'static str),

    // Error that occurs when we try to fetch a component from the builder but the given archetype mask is invalid
    DirectAccessArchetypeMissing(Mask, &'static str),

    // Error that occurs when we try to fetch a component from the builder but the given bundle index is invalid
    DirectAccessBundleIndexInvalid(usize, &'static str),
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
            QueryError::Unlinked(name) => write!(f, "Query of '{}' is not available in the query builder", name),
            QueryError::MutablyBorrowed(name) => write!(f, "Query of '{}' could not be borrowed because it is currently mutably borrowed", name),
            QueryError::DirectAccessArchetypeMissing(mask, name) => write!(
                f,
                "Component '{}' could not be accessed directly because the given archetype mask '{}' is invalid",
                name, mask
            ),
            QueryError::DirectAccessBundleIndexInvalid(index, name) => write!(
                f,
                "Component '{}' could not be accessed directly because the given bundle index '{}' is invalid",
                name, index
            ),
        }
    }
}

impl std::error::Error for QueryError {}
