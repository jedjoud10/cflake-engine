use crate::{ModelGPUObject, MaterialGPUObject, SubShaderGPUObject, ShaderGPUObject, ComputeShaderGPUObject, TextureGPUObject, TextureFillGPUObject, RendererGPUObject, interface, GPUObject};

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct GPUObjectID {
    pub index: Option<usize>,
}

impl GPUObjectID {    
    pub const None: Self = Self { index: None };
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
    /* #region Same thing but we don't have self, instead, we have a usize */
    pub fn usize_to_model<'a>(id: &'a usize) -> Option<&'a ModelGPUObject> {
        let gpuobject = interface::get_gpu_object_usize(id)?;
        if let GPUObject::Model(x) = gpuobject { Some(x) } else { None }
    }
    pub fn usize_to_material<'a>(id: &'a usize) -> Option<&MaterialGPUObject> {
        let gpuobject = interface::get_gpu_object_usize(id)?;
        if let GPUObject::Material(x) = gpuobject { Some(x) } else { None }
    }
    pub fn usize_to_subshader<'a>(id: &'a usize) -> Option<&'a SubShaderGPUObject> {
        let gpuobject = interface::get_gpu_object_usize(id)?;
        if let GPUObject::SubShader(x) = gpuobject { Some(x) } else { None }
    }
    pub fn usize_to_shader<'a>(id: &'a usize) -> Option<&'a ShaderGPUObject> {
        let gpuobject = interface::get_gpu_object_usize(id)?;
        if let GPUObject::Shader(x) = gpuobject { Some(x) } else { None }
    }
    pub fn usize_to_compute_shader<'a>(id: &'a usize) -> Option<&'a ComputeShaderGPUObject> {
        let gpuobject = interface::get_gpu_object_usize(id)?;
        if let GPUObject::ComputeShader(x) = gpuobject { Some(x) } else { None }
    }
    pub fn usize_to_texture<'a>(id: &'a usize) -> Option<&'a TextureGPUObject> {
        let gpuobject = interface::get_gpu_object_usize(id)?;
        if let GPUObject::Texture(x) = gpuobject { Some(x) } else { None }
    }
    pub fn usize_to_texture_fill<'a>(id: &'a usize) -> Option<&'a TextureFillGPUObject> {
        let gpuobject = interface::get_gpu_object_usize(id)?;
        if let GPUObject::TextureFill(x) = gpuobject { Some(x) } else { None }
    }
    pub fn usize_to_renderer<'a>(id: &'a usize) -> Option<&'a  RendererGPUObject> {
        let gpuobject = interface::get_gpu_object_usize(id)?;
        if let GPUObject::Renderer(x) = gpuobject { Some(x) } else { None }
    }
    /* #endregion */
    // Check if this GPU object ID is valid
    pub fn is_valid(&self) -> bool {
        interface::gpu_object_valid(self)
    }
}