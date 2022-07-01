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

impl std::error::Error for ResourceError {}

// Error that gets thrown whenever we fail to sort the event stages
pub enum StageError {
    CyclicReference,
    CyclicRuleReference(&'static str),
    MissingStage(&'static str, &'static str),
}

impl std::fmt::Debug for StageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StageError::CyclicReference => write!(
                f,
                "Detected a cyclic reference when trying to sort stages; aborting"
            ),
            StageError::CyclicRuleReference(name) => {
                write!(f, "Detcted a cyclic reference for rules of stage {name}")
            }
            StageError::MissingStage(current, name) => write!(
                f,
                "Stage {current} tried to reference stage {name}, but the latter does not exist"
            ),
        }
    }
}

impl std::fmt::Display for StageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for StageError {}
