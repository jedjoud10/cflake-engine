use crate::{
    BindResourceLayout, Buffer, GpuPod, Graphics, Id, IdVariant,
    ReflectedShader, Sampler, SetBindResourceError, Shader, Texel,
    Texture, TextureUsage, UniformBuffer, SetTextureError, SetBufferError, BufferUsage, SetBindGroupError,
};
use ahash::AHashMap;
use std::{marker::PhantomData, ops::RangeBounds, sync::Arc};

// A bind group allows us to set one or more bind entries to set them in the active render pass
// Bind groups are created using the set_bind_group method on the render pass
pub struct BindGroup<'a> {
    pub(crate) index: u32,
    pub(crate) reflected: Arc<ReflectedShader>,
    pub(crate) resources: Vec<wgpu::BindingResource<'a>>,
    pub(crate) slots: Vec<u32>,
    pub(crate) ids: Vec<Id>,
    pub(crate) _phantom: PhantomData<&'a ()>,
}

// Calculate the reflect bind group bitset of a specific reflect shader
pub(crate) fn calculate_refleced_group_bitset(
    shader: &ReflectedShader
) -> u32 {
    shader.bind_group_layouts
        .iter()
        .enumerate()
        .filter_map(|(index, val)| val.as_ref().map(|_| index))
        .fold(0u32, |current, offset| current | (1 << offset))
}


// Check if the user set the required bitset groups
// Returns Err(n) if the user did *not* set the value, with the specified index value returned as well
pub(crate) fn validate_set(
    needed: u32,
    set: u32,
) -> Result<(), u32> {
    if (set & needed) != needed {
        let missing = !set & needed;
        let missing = missing.leading_zeros();
        Err(missing)
    } else {
        Ok(())
    }
}

// Generate a new bind group from a callback (if needed)
// TODO: MUST FIX:
// currently requires the user to set the resources in the order they were defined
// uhhh no? checked and it seems fine. idk what you're on abt
pub(super) fn create_bind_group<'b>(
    graphics: &Graphics,
    modules: &[&str],
    reflected: Arc<ReflectedShader>,
    binding: u32,
    callback: impl FnOnce(&mut BindGroup<'b>),
) -> Result<Option<Arc<wgpu::BindGroup>>, SetBindGroupError> {
    // Check if the bind group index is supported
    if binding >= 4 {
        return Err(SetBindGroupError::BindGroupAdapterIndexInvalid(binding));
    }

    // Try to fetch the bind group layout from the reflected shader
    let bind_group_layout =
        reflected.bind_group_layouts.get(binding as usize).unwrap();

    // Don't do anything if the shader doesn't have this bind group
    let Some(bind_group_layout) = bind_group_layout else {
        return Ok(None);
    };

    // Pre-allocates vectors with the appropriate number of resources
    let count = bind_group_layout.bind_entry_layouts.len();
    let mut bind_group = BindGroup {
        _phantom: PhantomData,
        reflected: reflected.clone(),
        index: binding,
        resources: Vec::with_capacity(count),
        ids: Vec::with_capacity(count),
        slots: Vec::with_capacity(count),
    };
    callback(&mut bind_group);

    // Create the bind group that the user will interact with
    let BindGroup::<'_> {
        reflected,
        resources,
        slots,
        ids,
        ..
    } = bind_group;

    // Check the cache for a bind group with the given resources
    // If we do not find a bind group with the valid parametrs the nwe will create a new one and cache it instead
    let cache = &graphics.0.cached;
    let bind_group = match cache.bind_groups.entry(ids.clone()) {
        dashmap::mapref::entry::Entry::Occupied(occupied) => {
            occupied.get().clone()
        }
        dashmap::mapref::entry::Entry::Vacant(vacant) => {
            log::warn!("Did not find cached bind group (set = {binding}) {:?}, creating new one...", modules);

            // Get the bind group layout of the bind group
            let layout = &reflected.bind_group_layouts
                [binding as usize]
                .as_ref()
                .unwrap();
            let layout = graphics
                .0
                .cached
                .bind_group_layouts
                .get(layout)
                .unwrap();

            // Keep track of the resources we will set
            let mut set = 0u32;

            // Get the bind group entries
            let entries = resources
                .into_iter()
                .zip(slots.into_iter())
                .map(|(resource, binding)| {
                    set |= 1 << binding;
                    wgpu::BindGroupEntry {
                        binding,
                        resource,
                    }
                })
                .collect::<Vec<_>>();

            // Make sure we set ALL the required resources
            let reflected = reflected.bind_group_layouts[binding as usize]
                .as_ref()
                .unwrap()
                .bind_entry_layouts
                .iter()
                .map(|x| x.binding)
                .fold(0u32, |a, b| a | 1 << b);

            // Handle missing resources
            if let Err(index) = validate_set(reflected, set) {
                return Err(SetBindGroupError::MissingResource(index));
            }
            

            // Create a bind group descriptor of the entries
            let desc = wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &entries,
            };

            // Create the bind group and cache it for later use
            let bind_group =
                graphics.device().create_bind_group(&desc);
            let bind_group = Arc::new(bind_group);
            vacant.insert(bind_group.clone());
            bind_group
        }
    };
    Ok(Some(bind_group))
}

impl<'a> BindGroup<'a> {
    // Get the entry layout for a specific resource in this bind group
    // Returns None if there is no matching entry layout
    fn find_entry_layout<'c, 's>(
        index: u32,
        reflected: &'c ReflectedShader,
        name: &'s str,
    ) -> Result<&'c crate::BindResourceLayout, SetBindResourceError<'s>>
    {
        let groups = &reflected.bind_group_layouts;
        let (_, group) = groups
            .iter()
            .enumerate()
            .find(|(i, _)| *i == index as usize)
            .unwrap();
        let group = group.as_ref().unwrap();
        group
            .bind_entry_layouts
            .iter()
            .find(|x| x.name == name)
            .ok_or(SetBindResourceError::ResourceNotDefined {
                name,
                group: index,
            })
    }

    // Set a texture that can be sampled inside shaders using it's sampler
    pub fn set_sampled_texture<'s, T: Texture>(
        &mut self,
        name: &'s str,
        texture: &'a T,
    ) -> Result<(), SetBindResourceError<'s>> {
        // Make sure it's a sampled texture
        if !texture.usage().contains(TextureUsage::SAMPLED) {
            return Err(SetBindResourceError::SetTexture(SetTextureError::MissingSampleUsage));
        }

        // Try setting a sampler appropriate for this texture
        let sampler = format!("{name}_sampler");
        self.set_sampler(&sampler, texture.sampler().unwrap());

        // Get the binding entry layout for the given texture
        let entry = Self::find_entry_layout(
            self.index,
            &self.reflected,
            name,
        )?;

        // Get values needed for the bind entry
        let id = texture.raw().global_id();
        let resource =
            wgpu::BindingResource::TextureView(texture.view().unwrap());

        // Save the bind entry for later
        self.resources.push(resource);
        self.ids.push(Id::new(id, IdVariant::Texture));
        self.slots.push(entry.binding);
        Ok(())
    }

    // Set a storage texture that we can write / read from / to
    pub fn set_storage_texture<'s, T: Texture>(
        &mut self,
        name: &'s str,
        texture: &'a T,
    ) -> Result<(), SetBindResourceError<'s>> {
        // Make sure it's a sampled texture
        if !texture.usage().contains(TextureUsage::STORAGE) {
            return Err(SetBindResourceError::SetTexture(SetTextureError::MissingStorageUsage));
        }

        // Get the binding entry layout for the given texture
        let entry = Self::find_entry_layout(
            self.index,
            &self.reflected,
            name,
        )?;

        // Get values needed for the bind entry
        let id = texture.raw().global_id();
        let resource =
            wgpu::BindingResource::TextureView(texture.view().unwrap());

        // Save the bind entry for later
        self.resources.push(resource);
        self.ids.push(Id::new(id, IdVariant::Texture));
        self.slots.push(entry.binding);
        Ok(())
    }

    // Set a texture sampler so we can sample textures within the shader
    pub fn set_sampler<'s, T: Texel>(
        &mut self,
        name: &'s str,
        sampler: Sampler<'a, T>,
    ) -> Result<(), SetBindResourceError<'s>> {
        // Get the binding entry layout for the given sampler
        let entry = Self::find_entry_layout(
            self.index,
            &self.reflected,
            name,
        )?;

        // Get values needed for the bind entry
        let id = sampler.raw().global_id();
        let sampler = sampler.raw();
        let resource = wgpu::BindingResource::Sampler(sampler);

        // Save the bind entry for later
        self.resources.push(resource);
        self.ids.push(Id::new(id, IdVariant::Sampler));
        self.slots.push(entry.binding);
        Ok(())
    }

    // Set a uniform buffer that we can read from within shaders
    pub fn set_uniform_buffer<'s, T: GpuPod>(
        &mut self,
        name: &'s str,
        buffer: &'a UniformBuffer<T>,
        bounds: impl RangeBounds<usize>,
    ) -> Result<(), SetBindResourceError<'s>> {
        // Get the binding entry layout for the given buffer
        let entry = Self::find_entry_layout(
            self.index,
            &self.reflected,
            name,
        )?;

        // Get the buffer binding bounds
        let binding = buffer
            .convert_bounds_to_binding(bounds)
            .ok_or(SetBindResourceError::SetBuffer(
                SetBufferError::InvalidRange(buffer.len())
            ))?;

        // Get values needed for the bind entry
        let id = buffer.raw().global_id();
        let resource = wgpu::BindingResource::Buffer(binding);

        // Save the bind entry for later
        self.resources.push(resource);
        self.ids.push(Id::new(id, IdVariant::Buffer));
        self.slots.push(entry.binding);
        Ok(())
    }

    // Set a storage buffer that we can write / read from / to
    pub fn set_storage_buffer<'s, T: GpuPod, const TYPE: u32>(
        &mut self,
        name: &'s str,
        buffer: &'a Buffer<T, TYPE>,
        bounds: impl RangeBounds<usize>,
    ) -> Result<(), SetBindResourceError<'s>> {
        // Make sure it's a storage buffer
        if !buffer.usage().contains(BufferUsage::STORAGE) {
            return Err(SetBindResourceError::SetBuffer(SetBufferError::MissingStorageUsage));
        }

        // Get the binding entry layout for the given buffer
        let entry = Self::find_entry_layout(
            self.index,
            &self.reflected,
            name,
        )?;

        // Get the buffer binding bounds
        let binding = buffer
            .convert_bounds_to_binding(bounds)
            .ok_or(SetBindResourceError::SetBuffer(
                SetBufferError::InvalidRange(buffer.len())
            ))?;

        // Get values needed for the bind entry
        let id = buffer.raw().global_id();
        let resource = wgpu::BindingResource::Buffer(binding);

        // Save the bind entry for later
        self.resources.push(resource);
        self.ids.push(Id::new(id, IdVariant::Buffer));
        self.slots.push(entry.binding);
        Ok(())
    }
}
