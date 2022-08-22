use std::{marker::PhantomData, time::Duration, hash::{Hash, Hasher, BuildHasher}, collections::hash_map::{DefaultHasher, Entry}};

use crate::{prelude::TexelFormat, context::Context, object::ToGlName};

// This is what we will use within a display as it's backing store
#[derive(PartialEq)]
pub enum DisplayTarget<'a> {
    // This specifies a single layer in a texture2D that we can write to
    TextureLayer2D {
        texture_name: &'a u32,
        size: &'a vek::Extent2<u32>,
        layer: &'a u8,
        texel: TexelFormat,
    },

    // This specifies a render buffer target
    RenderBuffer2D {
        render_buffer_name: &'a u32,
        size: &'a vek::Extent2<u32>,
        texel: TexelFormat,
    }
}

impl Hash for DisplayTarget<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);

        match self {
            DisplayTarget::TextureLayer2D { texture_name, size, layer, texel } => {
                texture_name.hash(state);
                layer.hash(state);
                texel.hash(state);                
            },
            DisplayTarget::RenderBuffer2D { render_buffer_name, size, texel } => {
                render_buffer_name.hash(state);
                texel.hash(state);
            },
        }
    }
}

// This trait will be implemented for Texture2Ds and their underlying MipMapMut writers
pub trait ToDisplayTarget<'a> {
    fn into(&self) -> DisplayTarget<'a>; 
}

// This will be implemented for tuples that contain multiple ToDisplayTargets generics 
pub trait ViewportLayout<'a>: Hash {
    fn descriptors(&self) -> Vec<(TexelFormat, vek::Extent2<u16>)>;    
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
        let main_size = descriptors[0].1;

        // Check if the texel format and attachment sizes are both valid
        descriptors.iter().any(|(format, size)| {
            let format_valid = match format {
                TexelFormat::Depth => if !depth { depth = true; true } else { false },
                TexelFormat::Stencil => if !stencil { stencil = true; true } else { false },
                _ => true,
            };
            let size_valid = size == main_size;            
            !format_valid || !size_valid
        })
    }
}

// A display is a type of wrapper around raw OpenGL framebuffers
// Display have a specific rust lifetime, although they might/might not destroy their underlying framebuffer if needed
pub struct Viewport<'a, L: ViewportLayout<'a>> {
    lifetime_hint: RawFramebufferLifeHint,
    layout: L,
    size: vek::Extent2<u32>,
    name: u32,
}

impl<'a, L: ViewportLayout<'a>> Viewport<'a, L> {
    // This will create a display from the main window

    // This is called whenever we want to create a new display with a specific hint and layout
    pub fn new(context: &mut Context, layout: L, hint: RawFramebufferLifeHint) -> Option<Self> {
        // Check if the layout is even valid first
        if !layout.is_valid() {
            return None;
        }

        // Get el main display size
        let size = layout.descriptors()[0].1;

        // Hash the display layout to compare it
        let mut hasher = DefaultHasher::default();
        layout.hash(&mut hasher);
        let uid = hasher.finish();

        // Check if the context contains an underlying framebuffer with the same layout/attachments
        match context.framebuffers.entry(uid) {
            Entry::Occupied(old_fb_name) => {
                // We have a framebuffer that we can re-use
                return Some(Self {
                    lifetime_hint: hint,
                    layout,
                    size,
                    name: old_fb_name.get(),
                })
            },
            Entry::Vacant(vacant) => {
                // We don't have a framebuffer we can re-use, so we have to make a new one from scratch
            },
        }
    }

    // Get the size of the display (since we know it's valid this function will never fail)
    pub fn size(&self) -> vek::Extent2<u32> {
        self.size
    }

    // Get the lifetime hint for the underlying framebuffer
    pub fn hint(&self) -> RawFramebufferLifeHint {
        self.lifetime_hint
    }
}

impl<'a, L: ViewportLayout<'a>> ToGlName for Viewport<'a, L> {
    fn name(&self) -> u32 {
        self.name
    }
}

// This tells the main graphics context when it should delete a display's underlying framebuffer 
pub enum RawFramebufferLifeHint {
    WhenDropped,
    InDuration(Duration),
}

impl Default for RawFramebufferLifeHint {
    fn default() -> Self {
        Self::InDuration(Duration::from_secs(1))
    }
}