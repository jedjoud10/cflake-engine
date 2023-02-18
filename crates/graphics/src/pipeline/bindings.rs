use std::marker::PhantomData;

use crate::{Texture, GpuPod, UniformBuffer};

// This struct allows us to send variables to the currently used pipeline shader
// We use a lifetime to guarantee that the objects that we use live longer than the active pipeline itself
pub struct Bindings<'a> {
    pub(crate) _phantom: PhantomData<&'a ()>,
}

impl<'a> Bindings<'a> { 
    // Get a specific bind group to edit
    pub fn bind_group(&mut self, group: u32) -> BindGroup<'a> {
        BindGroup {
            group,
            _phantom: PhantomData,
        }
    }
}

// A unique bind group (descriptor set) that contains multiple
// textures, buffers, and push constants (even though push constants aren't stored within it)
pub struct BindGroup<'a> {
    group: u32,
    _phantom: PhantomData<&'a ()>,    
}

impl<'a> BindGroup<'a> {
    // Set a texture to be sampled
    pub fn set_sampler<T>(&'a self, value: &'a T) {
    }

    // Set a uniform buffer that we can read from within shaders
    pub fn set_buffer<T: GpuPod>(&mut self, buffer: &'a UniformBuffer<T>) {
        let entry = wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.raw().as_entire_binding(),
        };
    }
}