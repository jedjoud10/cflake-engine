use super::{ObjectID, PipelineObject, ReservedTrackedTaskID};
use crate::{
    advanced::{
        atomic::{AtomicGroup, AtomicGroupRead},
        compute::{ComputeShader, ComputeShaderExecutionSettings},
        shaderstorage::ShaderStorage,
    },
    basics::{
        material::Material,
        model::Model,
        readwrite::ReadBytes,
        renderer::Renderer,
        shader::{
            info::{ShaderInfo, ShaderInfoQuerySettings},
            Shader,
        },
        texture::Texture,
        transfer::Transfer,
        uniforms::ShaderUniformsGroup,
    },
    pipeline::camera::Camera,
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

// Pipeline object creation task
pub(crate) struct Construct<T: PipelineObject>(pub(crate) T, pub(crate) ObjectID<T>);