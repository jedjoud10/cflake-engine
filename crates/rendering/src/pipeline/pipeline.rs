use getset::Getters;
use others::Time;

use crate::{basics::{material::Material, mesh::Mesh, shader::Shader, texture::Texture}, advanced::{compute::ComputeShader, atomic::AtomicGroup, shader_storage::ShaderStorage}, utils::Window};

use super::PipelineCollection;


// Pipeline that mainly contains sets of specific objects like shaders and materials
#[derive(Getters)]
pub struct Pipeline {
    // OpenGL wrapper objects
    pub meshes: PipelineCollection<Mesh>,
    pub shaders: PipelineCollection< Shader>,
    pub compute_shaders: PipelineCollection<ComputeShader>,
    pub textures: PipelineCollection<Texture>,

    // Others
    pub materials: PipelineCollection<Material>,

    // Window
    pub window: Window,

    // Timings
    #[getset(get = "pub")]
    time: Time,
}