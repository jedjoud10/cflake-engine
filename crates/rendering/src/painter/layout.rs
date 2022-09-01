use crate::{prelude::{Texel, Texture2D, MipLayerMut, TexelFormat, UntypedTexel}, object::ToGlName};

// This is a wrapper around framebuffer attachments
// These values will be stored within the painter
#[derive(PartialEq, Clone, Copy)]
pub enum CanvasStorage {
    TextureLayer2D {
        texture_name: u32,
        level: u8,
        untyped: UntypedTexel
    },
}

// This trait is implemented for Textures and RenderBuffers that can be converted to CanvasStorage
pub trait ToCanvasStorage<'a> {
    fn into(&self) -> CanvasStorage;
}

// Implemented for tuples that we will use within ScopedPainters
pub trait CanvasLayout<'a> {
    // Get a list of all the canvas storages
    fn storages(&self) -> Vec<CanvasStorage>;

    // Check if the layout is valid
    fn valid(&self) -> bool {
        // Check if we have at maximum one depth and one stencil attachment
        let storages = self.storages();
        let mut depth = false;
        let mut stencil = false;

        // Check if the texel format and attachment sizes are both valid
        storages.iter().all(|storage| {
            let untyped = match storage {
                CanvasStorage::TextureLayer2D { texture_name, level, untyped } => untyped,
            };

            match untyped.enum_format {
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

// Will use the mip level given by the layer
impl<'a, T: Texel> ToCanvasStorage<'a> for MipLayerMut<'a, Texture2D<T>> {
    fn into(&self) -> CanvasStorage {
        CanvasStorage::TextureLayer2D {
            texture_name: self.texture().name(),
            level: self.level(),
            untyped: T::untyped(),
        }
    }
}
