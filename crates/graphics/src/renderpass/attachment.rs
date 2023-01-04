use vulkan::vk;
use crate::{LoadOp, Stencil, StoreOp, Texel, UntypedTexel, Texture2D, ColorTexel, Texture, UntypedLoadOp, ColorLayout, DepthStencilLayout, DepthElement};

// A color attachment that is passed to the render pass when starting it
pub trait ColorAttachments<'a, C: ColorLayout> {
}

// A depth stencil attachment that is passed to the render pass when starting it
pub trait DepthStencilAttachment<'a, DS: DepthStencilLayout + DepthElement> {
}

// A render target that can be used inside a renderpass (attachment)
// TODO: Handle MSAA maybe?
pub trait RenderTarget<'a, T: Texel> {
    // Get the untyped texel format
    fn untyped_texel() -> UntypedTexel;

    // Get the underlying image view that must be used for rendering
    fn image_view(&self) -> vk::ImageView;
}

// TODO: This should be implemented on the MipLevels instead of the texture  
impl<'a, T: Texel> RenderTarget<'a, T> for &'a mut Texture2D<T> {
    fn untyped_texel() -> UntypedTexel {
        T::untyped()
    }

    fn image_view(&self) -> vk::ImageView {
        self.view()
    }
}