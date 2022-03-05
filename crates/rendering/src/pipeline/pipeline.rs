use crate::{basics::{material::Material, mesh::Mesh, shader::Shader, texture::Texture}, advanced::{compute::ComputeShader, atomic::AtomicGroup, shader_storage::ShaderStorage}};

use super::PipelineCollection;


// Pipeline that mainly contains sets of specific objects like shaders and materials
pub struct Pipeline {
    // OpenGL wrapper objects
    pub meshes: PipelineCollection<Mesh>,
    pub shaders: PipelineCollection< Shader>,
    pub compute_shaders: PipelineCollection<ComputeShader>,
    pub textures: PipelineCollection<Texture>,

    // TODO: Specifiy
    pub materials: PipelineCollection<Material>,
}