use crate::{GpuPod, Shader, Texture, UniformBuffer};
use std::{marker::PhantomData, sync::Arc};

// A bind group allows us to set one or more bind entries to set them in the active render pass
// Bind groups are created using the set_bind_group method on the render pass
pub struct BindGroup<'a> {
    pub(crate) index: u32,
    pub(crate) shader: &'a Shader,
    pub(crate) resources: Vec<wgpu::BindingResource<'a>>,
    pub(crate) ids: Vec<wgpu::Id>,
    pub(crate) _phantom: PhantomData<&'a ()>,
}

impl<'a> BindGroup<'a> {
    // Get the entry layout for a specific resource in this bind group
    fn find_entry_layout(
        &mut self,
        name: &str,
    ) -> &crate::BindEntryLayout {
        let groups = &self.shader.vertex().reflected().groups;
        let (_, group) = groups
            .iter()
            .enumerate()
            .find(|(i, _)| *i == self.index as usize)
            .unwrap();
        group.entries.iter().find(|x| x.name == name).unwrap()
    }

    // Set a shader texture that can be sampled and red from
    pub fn set_sampler<T: Texture>(
        &'a mut self,
        name: &str,
        texture: &'a T,
    ) {
        // Get the binding entry layout for the given sampler
        let entry = self.find_entry_layout(name);

        /*
        match entry.binding_type {
            crate::BindingType::Sampler { sampler_binding } => todo!(),
        }

        // Get binding, id, view and resource needed for bind entry
        let binding = entry.binding;
        let id = texture.view().global_id();
        let view = texture.view();
        let resource = wgpu::BindingResource::TextureView(view);

        // Make a valid bind entry and locally store it
        let entry = BindEntry {
            binding,
            resource,
            id,
        };

        // Save the bind entry for later
        self.entries.push(entry);
        */
    }

    // Set a uniform buffer that we can read from within shaders
    pub fn set_buffer<T: GpuPod>(
        &mut self,
        name: &str,
        buffer: &'a UniformBuffer<T>,
    ) {
        // Get the binding entry layout for the given buffer
        let entry = self.find_entry_layout(name);

        // Make sure the layout is the same size as buffer stride
        match entry.binding_type {
            crate::BindingType::Buffer { size, .. } => {
                if (size as usize) != buffer.stride() {
                    panic!()
                }
            }
            _ => panic!(),
        }

        // Get values needed for the bind entry
        let binding = entry.binding;
        let id = buffer.raw().global_id();
        let buffer_binding = buffer.raw().as_entire_buffer_binding();
        let resource = wgpu::BindingResource::Buffer(buffer_binding);

        // Save the bind entry for later
        self.resources.push(resource);
        self.ids.push(id);
    }
}
