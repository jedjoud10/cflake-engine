use super::{tracked::TrackedTask, ConstructionTask, DeconstructionTask, ObjectID, ReservedTrackedID};

// A main pipeline task
pub(crate) enum PipelineTask {
    // Pipeline Objects
    Construction(ConstructionTask),
    Deconstruction(DeconstructionTask),

    Tracked(TrackedTask, ReservedTrackedID, Option<ReservedTrackedID>),
}
