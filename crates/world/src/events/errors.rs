
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
