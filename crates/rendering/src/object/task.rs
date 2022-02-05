use super::{tracked::TrackedTask, ConstructionTask, DeconstructionTask, ObjectID, ReservedTrackedID, UpdateTask};

// A main pipeline task
pub(crate) enum PipelineTask {
    // Pipeline Objects
    Construction(ConstructionTask),
    Deconstruction(DeconstructionTask),
    Update(UpdateTask),
    Tracked(TrackedTask, ReservedTrackedID, Option<ReservedTrackedID>),
}
