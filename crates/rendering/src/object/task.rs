use crate::{pipeline::{Pipeline, PipelineRenderer}};
use super::{tracked::TrackedTask, ConstructionTask, DeconstructionTask, ReservedTrackedID};

// A main pipeline task
pub(crate) enum PipelineTask {
    // Pipeline Objects
    Construction(ConstructionTask),
    Deconstruction(DeconstructionTask),
    Update(Box<dyn FnOnce(&mut Pipeline, &mut PipelineRenderer) + Send + Sync + 'static>),
    Tracked(TrackedTask, ReservedTrackedID, Option<ReservedTrackedID>),
}
