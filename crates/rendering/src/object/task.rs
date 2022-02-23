use super::{tracked::TrackedTask, ConstructionTask, DeconstructionTask, ReservedTrackedID};
use crate::pipeline::{Pipeline, SceneRenderer};

// A main pipeline task
pub(crate) enum PipelineTask {
    // Pipeline Objects
    Construction(ConstructionTask),
    Deconstruction(DeconstructionTask),
    Update(Box<dyn FnOnce(&mut Pipeline, &mut SceneRenderer) + Send + Sync + 'static>),
    Tracked(TrackedTask, ReservedTrackedID, Option<ReservedTrackedID>),
}
