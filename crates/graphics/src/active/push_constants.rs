use std::{marker::PhantomData, sync::Arc};
use crate::ReflectedShader;

// Push constants are tiny bits of memory that are going to get stored directly in a command encoder
// They are mostly used to upload bits of data very rapidly to use within shaders 
pub struct PushConstants<'a> {
    pub(crate) reflected: Arc<ReflectedShader>,
    pub(crate) offsets: Vec<u32>,
    pub(crate) data: Vec<Vec<u8>>,
    pub(crate) ids: Vec<wgpu::ShaderStages>,
    pub(crate) _phantom: PhantomData<&'a ()>,
}