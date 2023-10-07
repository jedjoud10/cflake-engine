use thiserror::Error;

use crate::StageId;

/// Error that gets thrown whenever we fail to sort the event stages
#[derive(Error, Debug)]
pub enum RegistrySortingError {
    #[error("Error while parsing Graph. Possibly due to cyclic reference / rules")]
    GraphVisitMissingNodes,

    #[error("Stage '{0:?}' tried to reference stage '{1:?}', but the latter stage does not exist")]
    MissingStage(StageId, StageId),
}

/// Error that gets thrown whenever we fail to create a valid stage
#[derive(Error, Debug)]
pub enum StageError {
    #[error("The given stage has an invalid name")]
    InvalidName,

    #[error("The given stage has no rules associated with it")]
    MissingRules,

    #[error("Tried to insert the stage into the pipeline, but the stage name was already used")]
    Overlapping,
}
