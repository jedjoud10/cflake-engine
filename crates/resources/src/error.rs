use crate::Resource;

// Error that gets thrown whenever we try to fetch a resource that doesn't exist or if we have overlapping handles
pub enum ResourceError {
    MissingResource(&'static str),
    Overlapping(&'static str),
}

impl ResourceError {
    // Create a new missing resource error from the resource type
    pub(crate) fn missing<T: Resource>() -> Self {
        Self::MissingResource(std::any::type_name::<T>())
    }

    // Create a new overlapping resource error
    pub(crate) fn overlapping<T: Resource>() -> Self {
        Self::Overlapping(std::any::type_name::<T>())
    }
}

impl std::fmt::Debug for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceError::MissingResource(name) => write!(f, "Tried fetching resource with name '{}', but it currently does not exist in the world", name),
            ResourceError::Overlapping(name) => write!(f, "Cannot access resource {} multiple times in the same get_mut() call", name),
        }
        
    }
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for ResourceError {

}