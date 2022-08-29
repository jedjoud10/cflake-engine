use std::{
    collections::hash_map::{DefaultHasher, Entry},
    hash::{Hash, Hasher},
    marker::PhantomData, time::Duration,
};

use crate::{
    context::Context,
    object::ToGlName,
    prelude::{MipLayerMut, Texel, TexelFormat, Texture, Texture2D},
};

use super::Display;

// This is what we will use within a canvas as it's backing store
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
            DisplayStorageDescriptor::TextureLayer2D {
                texture_name,
                size: _,
                level: layer,
                texel,
                _phantom,
            } => {
                texture_name.hash(state);
                layer.hash(state);
                texel.hash(state);
            }
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
        DisplayStorageDescriptor::TextureLayer2D {
            texture_name: self.name(),
            size: self.dimensions(),
            level: 0,
            texel: T::ENUM_FORMAT,
            _phantom: Default::default(),
        }
    }
}

// Will use the mip level given by the layer
impl<'a, T: Texel> ToDisplayStorageDescriptor<'a> for MipLayerMut<'a, Texture2D<T>> {
    fn into(&self) -> DisplayStorageDescriptor<'a> {
        DisplayStorageDescriptor::TextureLayer2D {
            texture_name: self.texture().name(),
            size: self.dimensions(),
            level: self.level(),
            texel: T::ENUM_FORMAT,
            _phantom: Default::default(),
        }
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
        if descriptors.is_empty() {
            return false;
        }

        // Check if we have at maximum one depth and one stencil attachment
        let mut depth = false;
        let mut stencil = false;

        // Check if the texel format and attachment sizes are both valid
        descriptors.iter().all(|desc| {
            match desc.texel() {
                TexelFormat::Depth => {
                    if !depth {
                        depth = true;
                        true
                    } else {
                        false
                    }
                }
                TexelFormat::Stencil => {
                    if !stencil {
                        stencil = true;
                        true
                    } else {
                        false
                    }
                }
                _ => true,
            }
        })
    }
}

// A viewport wrapper around raw OpenGL viewport
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    pub origin: vek::Vec2<u16>,
    pub extent: vek::Extent2<u16>,
}

// This is the raw framebuffer storage that will be stored in the context
pub(crate) struct RawFramebuffer {
    pub name: u32,
    pub current_countdown: Duration,
    pub countdown_reset_value: Duration,
}

// A canvas is a type of wrapper around raw OpenGL framebuffers
// Display have a specific rust lifetime, although they might/might not destroy their underlying framebuffer if needed
pub struct ScopedCanvas<'a, L: ScopedCanvasLayout<'a>> {
    layout: L,
    view: Viewport,
    name: u32,
    _phantom: PhantomData<&'a mut L>,
}

impl<'a, L: ScopedCanvasLayout<'a>> ScopedCanvas<'a, L> {
    // Create a default viewport without checking for safety (this is only called internally)
    pub fn new_default_unchecked<'b>(view: Viewport) -> ScopedCanvas<'b, ()> {
        ScopedCanvas {
            layout: (),
            view,
            name: 0,
            _phantom: Default::default(),
        }
    }

    // This is called whenever we want to create a new canvas with a specific hint and layout
    pub fn new(
        context: &mut Context,
        layout: L,
        view: Viewport,
    ) -> Option<Self> {
        // Check if the layout is even valid first
        if !layout.is_valid() {
            return None;
        }

        // Hash the canvas layout to compare it
        let hasher = DefaultHasher::default();
        let uid = layout
            .descriptors()
            .iter()
            .fold(hasher, |mut hash, target| {
                target.hash(&mut hash);
                hash
            })
            .finish();

        // Check if the context contains an underlying framebuffer with the same layout/attachments
        match context.framebuffers.entry(uid) {
            Entry::Occupied(mut old_fb_name) => {
                // Reset the countdown of the raw backing framebuffer
                let reset = old_fb_name.get().countdown_reset_value;
                old_fb_name.get_mut().current_countdown = reset;

                // Re-use the backing framebuffer
                return Some(Self {
                    layout,
                    view,
                    name: old_fb_name.get().name,
                    _phantom: Default::default(),
                });
            }
            Entry::Vacant(_vacant) => {
                // We don't have a framebuffer we can re-use, so we have to make a new one from scratch
                let name = unsafe {
                    let mut name = 0u32;
                    gl::CreateFramebuffers(1, &mut name);
                    gl::BindFramebuffer(gl::FRAMEBUFFER, name);
                    name
                };
        
                // Set the textures / render buffers
                let mut color_attachment_n = 0;
                for descriptor in layout.descriptors().iter() {
                    match descriptor {
                        DisplayStorageDescriptor::TextureLayer2D {
                            texture_name,
                            level,
                            texel, .. 
                        } => {
                            let attachment = match texel {
                                TexelFormat::Color | TexelFormat::GammaCorrectedColor => { 
                                    let attachment = gl::COLOR_ATTACHMENT0 + color_attachment_n;
                                    color_attachment_n += 1;
                                    attachment
                                },
                                TexelFormat::Depth => gl::DEPTH_ATTACHMENT,
                                TexelFormat::Stencil => gl::STENCIL_ATTACHMENT,
                            };

                            unsafe {
                                gl::NamedFramebufferTexture(name, attachment, *texture_name, *level as i32);
                            }
                        },
                    };
                }
        
                // Set the color attachment draw buffers
                unsafe {
                    let vec = (0..color_attachment_n).map(|i| gl::COLOR_ATTACHMENT0 + i).collect::<Vec<u32>>();
                    gl::NamedFramebufferDrawBuffers(name, color_attachment_n as i32, vec.as_ptr());
                }        
        
                // Check the framebuffer state
                unsafe {
                    let state = gl::CheckNamedFramebufferStatus(name, gl::FRAMEBUFFER);
                    if state != gl::FRAMEBUFFER_COMPLETE {
                        panic!("Framebuffer initialization error {state:X}");
                    }
                }    

                // Add the raw backing framebuffer to the context
                context.framebuffers.insert(uid, RawFramebuffer {
                    name,
                    current_countdown: Duration::from_secs(2),
                    countdown_reset_value: Duration::from_secs(2),
                });

                Some(Self {
                    layout,
                    view,
                    name,
                    _phantom: PhantomData::default(),
                })        
            }
        }
    }

    // Get the underlying viewport region
    pub fn viewport(&self) -> Viewport {
        self.view
    }
}

impl<'a, L: ScopedCanvasLayout<'a>> ToGlName for ScopedCanvas<'a, L> {
    fn name(&self) -> u32 {
        self.name
    }
}

// We can render to a scoped canvas, so it is in fact a canvas
impl<'a, L: ScopedCanvasLayout<'a>> Display for ScopedCanvas<'a, L> {
    fn viewport(&self) -> Viewport {
        self.view
    }

    fn name(&self) -> u32 {
        self.name
    }
}