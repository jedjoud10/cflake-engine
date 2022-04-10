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
