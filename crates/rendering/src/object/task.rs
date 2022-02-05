use super::{ConstructionTask, tracked::TrackedTask, ObjectID, DeconstructionTask, ReservedTrackedID};

// A main pipeline task 
pub(crate) enum PipelineTask {
    Construction(ConstructionTask),
    Deconstruction(DeconstructionTask),
    Tracked(TrackedTask, ReservedTrackedID, Option<ReservedTrackedID>),
}