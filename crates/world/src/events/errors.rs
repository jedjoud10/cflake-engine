use crate::StageKey;

// Error that gets thrown whenever we fail to sort the event stages
pub enum PipelineSortingError {
    CyclicReference,
    CyclicRuleReference(StageKey),
    MissingStage(StageKey, StageKey),
}

impl std::fmt::Debug for PipelineSortingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineSortingError::CyclicReference => write!(f, "Detected a cyclic reference when trying to sort stages; aborting"),
            PipelineSortingError::CyclicRuleReference(name) => {
                write!(f, "Detcted a cyclic reference for rules of stage '{name}'")
            }
            PipelineSortingError::MissingStage(current, name) => write!(f, "Stage '{current}' tried to reference stage '{name}', but the latter does not exist"),
        }
    }
}

impl std::fmt::Display for PipelineSortingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for PipelineSortingError {}

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
            StageError::Overlapping => write!(f, "Tried to insert the stage into the pipeline, but the stage name was already used"),
        }
    }
}

impl std::fmt::Display for StageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for StageError {}
