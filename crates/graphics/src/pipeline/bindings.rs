use std::{marker::PhantomData, sync::Arc};
use crate::{Texture, GpuPod, UniformBuffer, Shader};

pub struct BindEntry<'a> {
    pub(crate) binding: u32,
    pub(crate) resource: wgpu::BindingResource<'a>,
    pub(crate) id: wgpu::Id,
}

pub struct BindGroup<'a> {
    pub(crate) index: u32,
    pub(crate) shader: &'a Shader,
    pub(crate) entries: Vec<BindEntry<'a>>,
    pub(crate) _phantom: PhantomData<&'a ()>,    
}

impl<'a> BindGroup<'a> {
    // Set a shader texture that can be sampled and red from
    pub fn set_sampler<T: Texture>(&'a mut self, name: &str, texture: &'a T) {
        let groups = &self.shader.vertex().reflected().groups;
        let group = groups.iter().find(|x| x.index == self.index).unwrap();
        let entry = group.entries.iter().find(|x| x.name == name).unwrap();
        log::debug!("Found entry {:?} for group {}", entry, self.index);

        let binding = entry.binding;
        let id = texture.view().global_id();
        let view = texture.view();
        let resource = wgpu::BindingResource::TextureView(view);

        let entry = BindEntry {
            binding,
            resource,
            id,
        };

        self.entries.push(entry);
    }

    // Set a uniform buffer that we can read from within shaders
    pub fn set_buffer<T: GpuPod>(&mut self, name: &str, buffer: &'a UniformBuffer<T>) {
        let groups = &self.shader.vertex().reflected().groups;
        let group = groups.iter().find(|x| x.index == self.index).unwrap();
        let entry = group.entries.iter().find(|x| x.name == name).unwrap();
        log::debug!("Found entry {:?} for group {}", entry, self.index);

        let binding = entry.binding;
        let id = buffer.raw().global_id();
        let buffer_binding = buffer.raw().as_entire_buffer_binding();
        let resource = wgpu::BindingResource::Buffer(buffer_binding);

        let entry = BindEntry {
            binding,
            resource,
            id,
        };

        self.entries.push(entry);
    }
}