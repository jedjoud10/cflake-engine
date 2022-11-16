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
            RegistrySortingError::CyclicRuleReference((_, name)) => {
                write!(f, "Detcted a cyclic reference for rules of stage '{name}'")
            }
            RegistrySortingError::MissingStage((_, current), (_, name)) => write!(f, "Stage '{current}' tried to reference stage '{name}', but the latter does not exist"),
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
