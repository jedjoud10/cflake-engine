use crate::{prelude::{Texel, Texture2D, MipLayerMut, TexelFormat, UntypedTexel}, object::ToGlName};

// This is a wrapper around framebuffer attachments
// These values will be stored within the painter
#[derive(PartialEq)]
pub enum CanvasStorage {
    TextureLayer2D {
        texture_name: u32,
        level: u8,
        untyped: UntypedTexel
    },
}

// This trait is implemented for Textures and RenderBuffers that can be converted to CanvasStorage
pub trait ToCanvasStorage<'a> {
    fn into(self) -> CanvasStorage;
}

// Implemented for tuples that we will use within ScopedPainters
pub trait CanvasLayout<'a> {
    fn storages(self) -> Vec<CanvasStorage>;
}

// Will use the mip level given by the layer
impl<'a, T: Texel> ToCanvasStorage<'a> for MipLayerMut<'a, Texture2D<T>> {
    fn into(self) -> CanvasStorage {
        CanvasStorage::TextureLayer2D {
            texture_name: self.texture().name(),
            level: self.level(),
            untyped: T::untyped(),
        }
    }
}
