use super::{ObjectID, PipelineObject};
use crate::{
    advanced::{
        compute::ComputeShader,
        shader_storage::ShaderStorage, 
        atomic::AtomicGroup,
    },
    basics::{
        material::Material,
        model::Model,
        renderer::Renderer,
        shader::Shader,
        texture::Texture,
    }, pipeline::Pipeline,
};

// Task that we will send to the pipeline whenever we want to construct a specific pipeline object
pub(crate) enum ConstructionTask {
    Texture(Construct<Texture>),
    Material(Construct<Material>),
    Shader(Construct<Shader>),
    ComputeShader(Construct<ComputeShader>),
    Model(Construct<Model>),
    Renderer(Construct<Renderer>),
    AtomicGroup(Construct<AtomicGroup>),
    ShaderStorage(Construct<ShaderStorage>),
}
pub(crate) struct Construct<T: PipelineObject>(pub(crate) T, pub(crate) ObjectID<T>);

impl ConstructionTask {
    // Execute the construction task, running the "add()" method on our inner value
    pub(crate) fn execute(mut self, pipeline: &mut Pipeline) {
        match self {
            ConstructionTask::Texture(x) => Texture::add(x.0, pipeline, x.1).unwrap(),
            ConstructionTask::Material(x) => Material::add(x.0, pipeline, x.1).unwrap(),
            ConstructionTask::Shader(x) => Shader::add(x.0, pipeline, x.1).unwrap(),
            ConstructionTask::ComputeShader(x) => ComputeShader::add(x.0, pipeline, x.1).unwrap(),
            ConstructionTask::Model(x) => Model::add(x.0, pipeline, x.1).unwrap(),
            ConstructionTask::Renderer(x) => Renderer::add(x.0, pipeline, x.1).unwrap(),
            ConstructionTask::AtomicGroup(x) => AtomicGroup::add(x.0, pipeline, x.1).unwrap(),
            ConstructionTask::ShaderStorage(x) => ShaderStorage::add(x.0, pipeline, x.1).unwrap(),
        }       
    }
}