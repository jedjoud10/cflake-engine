use crate::{pipeline::Pipeline, basics::{texture::Texture, material::Material, shader::Shader, model::Model, renderer::Renderer}, advanced::{compute::ComputeShader, atomic::AtomicGroup, shaderstorage::ShaderStorage}};

use super::{ObjectID, ConstructionTask};

// Trait that is implemented on PipelineObjects that can be created and deleted
pub(crate) trait PipelineObject {
    // Reserve this object's ID, returning it's ID and itself
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> where Self: Sized; 

    // Send this object to the pipeline so it can be constructed using the add() function
    fn send(self, pipeline: &Pipeline, id: ObjectID<Self>) -> ConstructionTask;

    // Create this pipeline object using it's reserved object ID
    // This automatically adds it to the pipeline
    fn add(self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> where Self: Sized;
    
    // Delete this object, removing it from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> where Self: Sized;
}
