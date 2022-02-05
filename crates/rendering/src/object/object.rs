use crate::{pipeline::Pipeline};
use super::{ObjectID, ConstructionTask, DeconstructionTask};

// Trait that is implemented on PipelineObjects that can be created and deleted
pub(crate) trait PipelineObject where Self: Sized  {
    // Reserve this object's ID, returning it's ID and itself
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)>; 

    // Send this object to the pipeline so it can be constructed using the add() function
    fn send(self, pipeline: &Pipeline, id: ObjectID<Self>) -> ConstructionTask;

    // Create a deconstruction task so we can remove this object from the pipeline
    fn pull(pipeline: &Pipeline, id: ObjectID<Self>) -> DeconstructionTask;

    // Create this pipeline object using it's reserved object ID
    // This automatically adds it to the pipeline
    fn add(self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()>;
    
    // Delete this object, removing it from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self>;
}
