use crate::ComponentError;

pub enum QueryError {
    ComponentError(ComponentError),
    
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
        }
    }
}

impl std::error::Error for QueryError {}
