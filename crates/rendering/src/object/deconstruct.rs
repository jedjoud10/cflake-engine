use super::{ObjectID, PipelineObject};
use crate::{
    advanced::{atomic::AtomicGroup, compute::ComputeShader, shader_storage::ShaderStorage},
    basics::{
        lights::LightSource, material::Material, mesh::Mesh, renderer::Renderer, shader::Shader,
        texture::Texture,
    },
    pipeline::Pipeline,
};

// Task that we will send to the pipeline whenever we want to deconstruct a specific pipeline object
pub enum DeconstructionTask {
    Texture(Deconstruct<Texture>),
    Material(Deconstruct<Material>),
    Shader(Deconstruct<Shader>),
    ComputeShader(Deconstruct<ComputeShader>),
    Mesh(Deconstruct<Mesh>),
    Renderer(Deconstruct<Renderer>),
    AtomicGroup(Deconstruct<AtomicGroup>),
    ShaderStorage(Deconstruct<ShaderStorage>),
    LightSource(Deconstruct<LightSource>),
}
pub struct Deconstruct<T: PipelineObject>(pub(crate) ObjectID<T>);

impl DeconstructionTask {
    // Execute the deconstruction task, running the "delete()" method on our inner value
    pub(crate) fn execute(self, pipeline: &mut Pipeline) {
        match self {
            DeconstructionTask::Texture(x) => {
                Texture::delete(pipeline, x.0).unwrap();
            }
            DeconstructionTask::Material(x) => {
                Material::delete(pipeline, x.0).unwrap();
            }
            DeconstructionTask::Shader(x) => {
                Shader::delete(pipeline, x.0).unwrap();
            }
            DeconstructionTask::ComputeShader(x) => {
                ComputeShader::delete(pipeline, x.0).unwrap();
            }
            DeconstructionTask::Mesh(x) => {
                Mesh::delete(pipeline, x.0).unwrap();
            }
            DeconstructionTask::Renderer(x) => {
                Renderer::delete(pipeline, x.0).unwrap();
            }
            DeconstructionTask::AtomicGroup(x) => {
                AtomicGroup::delete(pipeline, x.0).unwrap();
            }
            DeconstructionTask::ShaderStorage(x) => {
                ShaderStorage::delete(pipeline, x.0).unwrap();
            }
            DeconstructionTask::LightSource(x) => {
                LightSource::delete(pipeline, x.0).unwrap();
            }
        }
    }
}
