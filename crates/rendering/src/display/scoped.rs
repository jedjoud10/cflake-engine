use std::{marker::PhantomData, time::{Duration, Instant}, hash::{Hash, Hasher, BuildHasher}, collections::hash_map::{DefaultHasher, Entry}};

use crate::{prelude::{TexelFormat, Texture2D, Texel, Texture, MipLayerMut}, context::Context, object::ToGlName};

use super::Display;

// This is what we will use within a display as it's backing store
// TODO: Remove the size and texel field from the enum variants and just put it in a main wrapper 
pub enum DisplayStorageDescriptor<'a> {
    // This specifies a single layer in a texture2D that we can write to
    TextureLayer2D {
        texture_name: u32,
        size: vek::Extent2<u16>,
        level: u8,
        texel: TexelFormat,
        _phantom: PhantomData<&'a mut u32>,
    },
}

impl<'a> DisplayStorageDescriptor<'a> {
    // Get the viewport size of self
    pub fn size(&self) -> vek::Extent2<u16> {
        match self {
            DisplayStorageDescriptor::TextureLayer2D { size, .. } => *size,
        }
    }
    
    // Get the texel format of self
    pub fn texel(&self) -> TexelFormat {
        match self {
            DisplayStorageDescriptor::TextureLayer2D { texel, .. } => *texel,
        }
    }
}

impl Hash for DisplayStorageDescriptor<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);

        match self {
            DisplayStorageDescriptor::TextureLayer2D { texture_name, size, level: layer, texel, _phantom } => {
                texture_name.hash(state);
                layer.hash(state);
                texel.hash(state);                
            },
        }
    }
}

// This trait will be implemented for Texture2Ds and their underlying MipMapMut writers
pub trait ToDisplayStorageDescriptor<'a> {
    fn into(&self) -> DisplayStorageDescriptor<'a>; 
}

// This will take the main miplevel
impl<'a, T: Texel> ToDisplayStorageDescriptor<'a> for &'a mut Texture2D<T> {
    fn into(&self) -> DisplayStorageDescriptor<'a> {
        DisplayStorageDescriptor::TextureLayer2D { texture_name: self.name(), size: self.dimensions(), level: 1, texel: T::ENUM_FORMAT, _phantom: Default::default() }
    }
}

// Will use the mip level given by the layer
impl<'a, T: Texel> ToDisplayStorageDescriptor<'a> for MipLayerMut<'a, Texture2D<T>> {
    fn into(&self) -> DisplayStorageDescriptor<'a> {
        DisplayStorageDescriptor::TextureLayer2D { texture_name: self.texture().name(), size: self.dimensions(), level: self.level(), texel: T::ENUM_FORMAT, _phantom: Default::default() }
    }
}

// This will be implemented for tuples that contain multiple ToDisplayTargets generics 
pub trait ScopedCanvasLayout<'a> {
    // Get a list of all the framebuffer attachments
    fn descriptors(&self) -> Vec<DisplayStorageDescriptor<'a>>;    

    // Check if the canvas is valid (at max 1 depth attachment, at max 1 stencil attachment, not empty)
    fn is_valid(&self) -> bool {
        // Check if the count is valid
        let descriptors = self.descriptors();
        if descriptors.len() == 0 {
            return false;
        }

        // Check if we have at maximum one depth and one stencil attachment
        let mut depth = false;
        let mut stencil = false;

        // Check if the sizes are all the same as well
        let main_size = descriptors[0].size();

        // Check if the texel format and attachment sizes are both valid
        descriptors.iter().any(|desc| {
            let format_valid = match desc.texel() {
                TexelFormat::Depth => if !depth { depth = true; true } else { false },
                TexelFormat::Stencil => if !stencil { stencil = true; true } else { false },
                _ => true,
            };
            let size_valid = desc.size() == main_size;            
            !format_valid || !size_valid
        })
    }
}

// A viewport wrapper around raw OpenGL viewport
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    pub origin: vek::Vec2<u16>,
    pub extent: vek::Extent2<u16>,
}


// A display is a type of wrapper around raw OpenGL framebuffers
// Display have a specific rust lifetime, although they might/might not destroy their underlying framebuffer if needed
pub struct ScopedCanvas<'a, L: ScopedCanvasLayout<'a>> {
    lifetime_hint: RawFramebufferLifeHint,
    layout: L,
    view: Viewport,
    name: u32,
    _phantom: PhantomData<&'a mut L>,
}

impl<'a, L: ScopedCanvasLayout<'a>> ScopedCanvas<'a, L> {
    // Create a default viewport without checking for safety (this is only called internally)
    pub fn new_default_unchecked<'b>(view: Viewport) -> ScopedCanvas<'b, ()> {
        ScopedCanvas { lifetime_hint: RawFramebufferLifeHint::NeverDelete, layout: (), view, name: 0, _phantom: Default::default() }
    }

    // This is called whenever we want to create a new display with a specific hint and layout
    pub fn new(context: &mut Context, layout: L, hint: RawFramebufferLifeHint, view: Viewport) -> Option<Self> {
        // Check if the layout is even valid first
        if !layout.is_valid() {
            return None;
        }

        // Hash the display layout to compare it
        let mut hasher = DefaultHasher::default();
        let uid = layout.descriptors().iter().fold(hasher, |mut hash, target| {
            target.hash(&mut hash);
            hash
        }).finish();

        // Check if the context contains an underlying framebuffer with the same layout/attachments
        let mut borrowed = context.framebuffers.borrow_mut();
        match borrowed.entry(uid) {
            Entry::Occupied(old_fb_name) => {
                // We have a framebuffer that we can re-use
                return Some(Self {
                    lifetime_hint: hint,
                    layout,
                    view,
                    name: old_fb_name.get().0,
                    _phantom: Default::default()
                })
            },
            Entry::Vacant(vacant) => {
                // We don't have a framebuffer we can re-use, so we have to make a new one from scratch
                todo!()
            },
        }
    }

    // Get the underlying viewport region
    pub fn viewport(&self) -> Viewport {
        self.view
    }

    // Get the lifetime hint for the underlying framebuffer
    pub fn hint(&self) -> RawFramebufferLifeHint {
        self.lifetime_hint
    }
}

impl<'a, L: ScopedCanvasLayout<'a>> ToGlName for ScopedCanvas<'a, L> {
    fn name(&self) -> u32 {
        self.name
    }
}

// We can render to a scoped canvas, so it is in fact a display
impl<'a, L: ScopedCanvasLayout<'a>> Display for ScopedCanvas<'a, L> {
    fn viewport(&self) -> Viewport {
        self.view
    }

    fn name(&self) -> u32 {
        self.name
    }
}

// This tells the main graphics context when it should delete a display's underlying framebuffer 
#[derive(Clone, Copy)]
pub enum RawFramebufferLifeHint {
    DeleteWhenDropped,
    NeverDelete,
}