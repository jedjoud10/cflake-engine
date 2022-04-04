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
    NotLinked(&'static str),
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
            QueryError::NotLinked(name) => write!(f, "Query of '{}' is not available in the query builder", name),
        }
    }
}

impl std::error::Error for QueryError {}
