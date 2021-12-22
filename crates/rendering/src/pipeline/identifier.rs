use crate::{ModelGPUObject, MaterialGPUObject, SubShaderGPUObject, ShaderGPUObject, ComputeShaderGPUObject, TextureGPUObject, TextureFillGPUObject, RendererGPUObject, interface, GPUObject};

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct GPUObjectID {
    pub index: Option<usize>,
}

impl GPUObjectID {    
    /* #region Get each enum variant from a GPU object */
    pub fn to_model<'a>(&'a self) -> Option<&'a ModelGPUObject> {
        let gpuobject = interface::get_gpu_object(self)?;
        if let GPUObject::Model(x) = gpuobject { Some(x) } else { None }
    }
    pub fn to_material<'a>(&self) -> Option<&MaterialGPUObject> {
        let gpuobject = interface::get_gpu_object(self)?;
        if let GPUObject::Material(x) = gpuobject { Some(x) } else { None }
    }
    pub fn to_subshader<'a>(&'a self) -> Option<&'a SubShaderGPUObject> {
        let gpuobject = interface::get_gpu_object(self)?;
        if let GPUObject::SubShader(x) = gpuobject { Some(x) } else { None }
    }
    pub fn to_shader<'a>(&'a self) -> Option<&'a ShaderGPUObject> {
        let gpuobject = interface::get_gpu_object(self)?;
        if let GPUObject::Shader(x) = gpuobject { Some(x) } else { None }
    }
    pub fn to_compute_shader<'a>(&'a self) -> Option<&'a ComputeShaderGPUObject> {
        let gpuobject = interface::get_gpu_object(self)?;
        if let GPUObject::ComputeShader(x) = gpuobject { Some(x) } else { None }
    }
    pub fn to_texture<'a>(&'a self) -> Option<&'a TextureGPUObject> {
        let gpuobject = interface::get_gpu_object(self)?;
        if let GPUObject::Texture(x) = gpuobject { Some(x) } else { None }
    }
    pub fn to_texture_fill<'a>(&'a self) -> Option<&'a TextureFillGPUObject> {
        let gpuobject = interface::get_gpu_object(self)?;
        if let GPUObject::TextureFill(x) = gpuobject { Some(x) } else { None }
    }
    pub fn to_renderer<'a>(&'a self) -> Option<&'a  RendererGPUObject> {
        let gpuobject = interface::get_gpu_object(self)?;
        if let GPUObject::Renderer(x) = gpuobject { Some(x) } else { None }
    }
    /* #endregion */
    // Check if this GPU object ID is valid
    pub fn is_valid(&self) -> bool {
        interface::gpu_object_valid(self)
    }
}