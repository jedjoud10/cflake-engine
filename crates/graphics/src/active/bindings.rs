use ahash::AHashMap;
use thiserror::Error;
use crate::{GpuPod, Shader, Texture, UniformBuffer, ReflectedShader, Sampler, Texel, ValueFiller, StructMemberLayout, BindEntryLayout, FillError, GpuPodRelaxed};
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
    },
}

// A bind group allows us to set one or more bind entries to set them in the active render pass
// Bind groups are created using the set_bind_group method on the render pass
pub struct BindGroup<'a> {
    pub(crate) index: u32,
    pub(crate) reflected: Arc<ReflectedShader>,
    pub(crate) resources: Vec<wgpu::BindingResource<'a>>,
    pub(crate) fill_ubos: Vec<(Vec<u8>, BindEntryLayout)>,
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

    // Fetches an already allocated uniform buffer that we can fill up with data
    pub fn fill_ubo<'s>(
        &mut self,
        name: &'s str,
        callback: impl FnOnce(&mut FillBuffer)
    ) -> Result<(), BindError<'s>> {
        // Get the binding entry layout for the given buffer
        let entry = Self::find_entry_layout(
            self.index,
            &self.reflected,
            name
        )?;
        
        // Pre-allocate a vector with an appropriate size
        let (size, members) = match entry.binding_type {
            crate::BindingType::Buffer { size, ref members, .. } => (size as usize, members),
            _ => panic!(),
        };

        // Le vecteur that contains le data
        let mut vector = vec![0u8; size];

        // Create the fill buffer
        let mut fill_buffer = FillBuffer {
            data: &mut vector,
            members: members.as_slice(),
            _phantom: PhantomData,
        };
        
        // Execute the call back to set the UBO fields
        callback(&mut fill_buffer);
        drop(fill_buffer);

        // Set the fill UBO data
        self.fill_ubos.push((vector, entry.clone()));
        Ok(())
    }
}

// A fill buffer can be used to fill UBO data field by field instead of uploading raw bytes to the GPU
// All it does is fetch field layout, write to a byte buffer, then upload when the bind group gets dropped
pub struct FillBuffer<'a> {
    data: &'a mut [u8],
    members: &'a [StructMemberLayout],
    _phantom: PhantomData<&'a ()>
}

impl ValueFiller for FillBuffer<'_> {
    // Set the value of a UBO field
    fn set<'s, T: GpuPodRelaxed>(&mut self, name: &'s str, value: T) -> Result<(), crate::FillError<'s>> {
        // Get the struct member layout for the proper field
        let valid = self.members
            .iter()
            .filter(|member| member.name == name)
            .next();

        // Return early if we don't have a field to set
        let Some(valid) = valid else {
            return Err(FillError::MissingField { name: name });
        };

        // Convert the data to a byte slice
        let value = [value];
        let bytes = bytemuck::cast_slice::<T, u8>(&value);

        // Set the value within the pre-allocated memory
        let offset = valid.offset as usize;
        let size = valid.size as usize;
        let out = &mut self.data[offset..][..size];
        out.copy_from_slice(bytes);

        Ok(())
    }
}
