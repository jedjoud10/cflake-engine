use super::{ConstructionTask, DeconstructionTask, ObjectID};
use crate::pipeline::Pipeline;

// Trait that is implemented on PipelineObjects that can be created and deleted
pub trait PipelineObject
where
    Self: Sized,
{
    // Should we update the pipeline state if Self is mutated?
    const UPDATE: bool = false;
    
    // Reserve this object's ID, returning it's ID and itself
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)>;

    // Send this object to the pipeline so it can be constructed using the add() function
    fn send(self, id: ObjectID<Self>) -> ConstructionTask;

    // Create a deconstruction task so we can remove this object from the pipeline
    fn pull(id: ObjectID<Self>) -> DeconstructionTask;

    // Called whenever we mutate a specific object
    // Useful whenever we must notify the pipeline that something changed, and that we must update
    // This is also called by default whenever we add an object to the pipeline
    fn mutated(&self) {}

    // Create this pipeline object using it's reserved object ID
    // This automatically adds it to the pipeline
    fn add(self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()>;

    // Delete this object, removing it from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self>;
}
