use crate::{
    object::{ToGlName, ToGlTarget},
    prelude::{Texel, Texture, Texture2D, TexelFormat},
};

// Attachment descriptions are used to identify textures and their format
pub struct AttachmentDescription {
    pub(crate) name: u32,
    pub(crate) format: TexelFormat,
}

impl AttachmentDescription {
    // Create an attachment description from a texture 2D
    pub(crate) fn new<T: Texel>(texture: &Texture2D<T>) -> Self {
        Self { name: texture.name(), format: T::ENUM_FORMAT }
    }
}

// The canvas layout will be implemented for canvas attachment tuples (color layout, special layout)
pub trait CanvasLayout {
    fn resize(&mut self, size: vek::Extent2<u16>);
    fn attachments(&self) -> Vec<AttachmentDescription>;

    // Check if we can create a new canvas
    fn is_instantiable(&self) -> bool {
        let attachments = self.attachments();
        let mut depth_enabled = false;
        let mut stencil_enabled = false;

        for i in attachments {
            match i.format {
                // Make sure there is only one or zero depth attachments
                TexelFormat::Depth => if !depth_enabled {
                    depth_enabled = true;
                } else {
                    return false;
                },

                // Make suret here is only one or zero stencil attachments
                TexelFormat::Stencil => if !stencil_enabled {
                    stencil_enabled = true;
                } else {
                    return false;
                },

                // Don't care
                TexelFormat::Color | TexelFormat::GammaCorrectedColor => {}
            }
        }

        true
    }
}