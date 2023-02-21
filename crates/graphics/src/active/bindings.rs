use thiserror::Error;
use crate::{GpuPod, Shader, Texture, UniformBuffer, ReflectedShader, Sampler, Texel, ValueFiller};
use std::{marker::PhantomData, sync::Arc};

#[derive(Debug, Error)]
pub enum BindError<'a> {
    #[error("The bind resource '{name}' at bind group '{group}' was not defined")]
    ResourceNotDefined {
        name: &'a str,
        group: u32,
    },

    #[error("The given buffer at '{name}' has different strides (layout stride = {defined}, buffer stride = {inputted})")]
    BufferDifferentStride {
        name: &'a str,
        defined: usize,
        inputted: usize,
    },

    #[error("The texutre '{name}' does not have a correspodning sampler named '{name}_sampler'")]
    TextureMissingSampler {
        name: &'a str
    }
}

// A bind group allows us to set one or more bind entries to set them in the active render pass
// Bind groups are created using the set_bind_group method on the render pass
pub struct BindGroup<'a> {
    pub(crate) index: u32,
    pub(crate) reflected: Arc<ReflectedShader>,
    pub(crate) resources: Vec<wgpu::BindingResource<'a>>,
    pub(crate) slots: Vec<u32>,
    pub(crate) ids: Vec<wgpu::Id>,
    pub(crate) _phantom: PhantomData<&'a ()>,
}

impl<'a> BindGroup<'a> {
    // Get the entry layout for a specific resource in this bind group
    // Returns None if there is no matching entry layout 
    fn find_entry_layout<'c, 's>(
        index: u32,
        reflected: &'c ReflectedShader,
        name: &'s str,
    ) -> Result<&'c crate::BindEntryLayout, BindError<'s>> {
        let groups = &reflected.bind_group_layouts;
        let (_, group) = groups
            .iter()
            .enumerate()
            .find(|(i, _)| *i == index as usize)
            .unwrap();
        let group = group.as_ref().unwrap();
        group.bind_entry_layouts.iter().find(|x| x.name == name).ok_or(BindError::ResourceNotDefined {
            name,
            group: index
        })
    }

    // Set a texture that can be read / sampler with the help of a sampler
    pub fn set_texture<'s, T: Texture>(
        &mut self,
        name: &'s str,
        texture: &'a T,
    ) -> Result<(), BindError<'s>> {
        // Get the binding entry layout for the given texture
        let entry = Self::find_entry_layout(
            self.index,
            &self.reflected,
            name
        )?;

        // Get values needed for the bind entry
        let id = texture.raw().global_id();
        let resource = wgpu::BindingResource::TextureView(texture.view());

        // Save the bind entry for later
        self.resources.push(resource);
        self.ids.push(id);
        self.slots.push(entry.binding);
        Ok(())
    }

    // Set the texture sampler so we can sample textures within the shader
    pub fn set_sampler<'s, T: Texel>(
        &mut self,
        name: &'s str,
        sampler: Sampler<'a, T>,
    ) -> Result<(), BindError<'s>> {
        // Get the binding entry layout for the given sampler
        let entry = Self::find_entry_layout(
            self.index,
            &self.reflected,
            name
        )?;

        // Get values needed for the bind entry
        let id = sampler.raw().global_id();
        let sampler = sampler.raw();
        let resource = wgpu::BindingResource::Sampler(sampler);

        // Save the bind entry for later
        self.resources.push(resource);
        self.ids.push(id);
        self.slots.push(entry.binding);
        Ok(())
    }

    // Set a uniform buffer that we can read from within shaders
    pub fn set_buffer<'s, T: GpuPod>(
        &mut self,
        name: &'s str,
        buffer: &'a UniformBuffer<T>,
    ) -> Result<(), BindError<'s>> {
        // Get the binding entry layout for the given buffer
        let entry = Self::find_entry_layout(
            self.index,
            &self.reflected,
            name
        )?;

        // Make sure the layout is the same size as buffer stride
        match entry.binding_type {
            crate::BindingType::Buffer { size, .. } => {
                if (size as usize) != buffer.stride() {
                    return Err(BindError::BufferDifferentStride {
                        name,
                        defined: size as usize,
                        inputted: buffer.stride()
                    });
                }
            }
            _ => panic!(),
        }

        // Get values needed for the bind entry
        let id = buffer.raw().global_id();
        let buffer_binding = buffer.raw().as_entire_buffer_binding();
        let resource = wgpu::BindingResource::Buffer(buffer_binding);

        // Save the bind entry for later
        self.resources.push(resource);
        self.ids.push(id);
        self.slots.push(entry.binding);
        Ok(())
    }

    // Create a new uniform buffer and fills it's fields one by one
    // This is basically emulates OpenGL uniforms through UBOs, should only be used for data
    // that doesn't change too much and that is unique to each material / draw batch
    pub fn fill_buffer<'s>(
        &mut self,
        name: &'s str,
        callback: impl FnOnce(&mut FillBuffer)
    ) -> Result<(), BindError<'s>> {
        /*
        // Get the binding entry layout for the given buffer
        let entry = Self::find_entry_layout(
            self.index,
            &self.reflected,
            name
        )?;
        */
        Ok(())
    }
}


pub struct FillBuffer<'a> {
    _phantom: PhantomData<&'a ()>
}

impl ValueFiller for FillBuffer<'_> {
}
