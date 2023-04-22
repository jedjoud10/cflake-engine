use crate::StageId;

// Error that gets thrown whenever we fail to sort the event stages
pub enum RegistrySortingError {
    CyclicReference,
    CyclicRuleReference(StageId),
    MissingStage(StageId, StageId),
}

impl std::fmt::Debug for RegistrySortingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistrySortingError::CyclicReference => write!(f, "Detected a cyclic reference when trying to sort stages"),
            RegistrySortingError::CyclicRuleReference(id) => {
                write!(f, "Detected a cyclic reference for rules of event '{}' from system '{}'", id.caller.name, id.system.name)
            }
            RegistrySortingError::MissingStage(current, other) => write!(f, 
                "Stage '{}' from system '{}' tried to reference stage '{}' from system '{}', but the latter stage does not exist",
                current.caller.name, current.system.name, other.caller.name, other.system.name
            ),
        }
    }
}

impl std::fmt::Display for RegistrySortingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for RegistrySortingError {}

// Error that gets thrown whenever we fail to create a valid stage
pub enum StageError {
    InvalidName,
    MissingRules,
    Overlapping,
}

impl std::fmt::Debug for StageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StageError::InvalidName => write!(f, "The given stage has an invalid name"),
            StageError::MissingRules => {
                write!(f, "The given stage has no rules associated with it")
            }
            StageError::Overlapping => write!(
                f,
                "Tried to insert the stage into the pipeline, but the stage name was already used"
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
