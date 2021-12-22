use crate::{
    interface, ComputeShaderGPUObject, GPUObject, MaterialGPUObject, ModelGPUObject, RendererGPUObject, ShaderGPUObject, SubShaderGPUObject, TextureFillGPUObject, TextureGPUObject,
};

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq)]
// Just a simple ID stored in each GPU object, that way we can save a bit of memory when dealing with GPU objects that do not have any functions
pub struct GPUObjectID {
    pub index: Option<usize>,
}

impl GPUObjectID {
    pub const None: Self = Self { index: None };
}
