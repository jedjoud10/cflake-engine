use crate::{ModelGPUObject, MaterialGPUObject, SubShaderGPUObject, ShaderGPUObject, ComputeShaderGPUObject, TextureGPUObject, TextureFillGPUObject, RendererGPUObject, interface, GPUObject};

#[derive(Default, Clone)]
pub struct GPUObjectID {
    pub index: Option<usize>,
}

impl GPUObjectID {    
    /* #region Get each enum variant from a GPU object */
    pub fn to_model(&self) -> Option<ModelGPUObject> {
        let gpuobject = interface::get_gpu_object(self)?;
        if let GPUObject::Model(x) = gpuobject { Some(x) } else { None }
    }
    pub fn to_material(&self) -> Option<MaterialGPUObject> {
        let gpuobject = interface::get_gpu_object(self)?;
        if let GPUObject::Material(x) = gpuobject { Some(x) } else { None }
    }
    pub fn to_subshader(&self) -> Option<SubShaderGPUObject> {
        let gpuobject = interface::get_gpu_object(self)?;
        if let GPUObject::SubShader(x) = gpuobject { Some(x) } else { None }
    }
    pub fn to_shader(&self) -> Option<ShaderGPUObject> {
        let gpuobject = interface::get_gpu_object(self)?;
        if let GPUObject::Shader(x) = gpuobject { Some(x) } else { None }
    }
    pub fn to_compute_shader(&self) -> Option<ComputeShaderGPUObject> {
        let gpuobject = interface::get_gpu_object(self)?;
        if let GPUObject::ComputeShader(x) = gpuobject { Some(x) } else { None }
    }
    pub fn to_texture(&self) -> Option<TextureGPUObject> {
        let gpuobject = interface::get_gpu_object(self)?;
        if let GPUObject::Texture(x) = gpuobject { Some(x) } else { None }
    }
    pub fn to_texture_fill(&self) -> Option<TextureFillGPUObject> {
        let gpuobject = interface::get_gpu_object(self)?;
        if let GPUObject::TextureFill(x) = gpuobject { Some(x) } else { None }
    }
    pub fn to_renderer(&self) -> Option<RendererGPUObject> {
        let gpuobject = interface::get_gpu_object(self)?;
        if let GPUObject::Renderer(x) = gpuobject { Some(x) } else { None }
    }
    /* #endregion */
}