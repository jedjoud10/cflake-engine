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
            entries: Vec::new(),
        }
    }
}

// A unique bind group (descriptor set) that contains multiple
// textures, buffers, and push constants (even though push constants aren't stored within it)
pub struct BindGroup<'a> {
    group: u32,
    entries: Vec<wgpu::BindGroupEntry<'a>>,
    _phantom: PhantomData<&'a ()>,    
}

impl<'a> BindGroup<'a> {
    // Set a shader texture that can be sampled and red from
    pub fn set_sampler<T: Texture>(&'a mut self, name: &str, texture: &'a T) {
        let entry = wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(texture.view()),
        };

        self.entries.push(entry);
    }

    // Set a uniform buffer that we can read from within shaders
    pub fn set_buffer<T: GpuPod>(&mut self, name: &str, buffer: &'a UniformBuffer<T>) {
        let entry = wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(buffer.raw().as_entire_buffer_binding()),
        };

        self.entries.push(entry);
    }
}