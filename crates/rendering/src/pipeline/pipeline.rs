use slotmap::SlotMap;

use crate::{basics::{material::Material, mesh::Mesh, shader::Shader, texture::Texture}, advanced::compute::ComputeShader};

slotmap::new_key_type! {
    pub struct MaterialKey;
    pub struct MeshKey;
    pub struct ShaderKey;
    pub struct ComputeShaderKey;
    pub struct TextureKey;
}


// Pipeline that mainly contains sets of specific objects like shaders and materials
pub struct Pipeline {
    pub materials: SlotMap<MaterialKey, Material>,
    pub meshes: SlotMap<MeshKey, Mesh>,
    pub shaders: SlotMap<ShaderKey, Shader>,
    pub compute_shaders: SlotMap<ComputeShaderKey, ComputeShader>,
    pub textures: SlotMap<TextureKey, Texture>,
}