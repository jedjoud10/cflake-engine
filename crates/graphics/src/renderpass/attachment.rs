use std::marker::PhantomData;

use crate::{
    ColorLayout, ColorTexel, DepthElement, DepthStencilLayout,
    LoadOp, Stencil, StoreOp, Texel, Texture, Texture2D,
    UntypedLoadOp, UntypedTexel,
};
use crate::vulkan::vk;

// A color attachment that is passed to the render pass when starting it
pub trait ColorAttachments<'a, C: ColorLayout> {
    fn image_views(&self) -> Vec<vk::ImageView>;
}

// A depth stencil attachment that is passed to the render pass when starting it
pub trait DepthStencilAttachment<'a, DS: DepthStencilLayout> {}
impl<'a> DepthStencilAttachment<'a, ()> for () {}

// A render target that can be used inside a renderpass (attachment)
pub struct RenderTarget<'a, T: Texel> {
    image: vk::Image,
    view: vk::ImageView,
    _phantom: PhantomData<T>,
    _phantom2: PhantomData<&'a ()>,
}

impl<'a, T: Texel> RenderTarget<'a, T> {
    // Create a render target from the raw Vulkan parts
    // This assumes that the image is valid to be used as a render target
    pub unsafe fn from_raw_parts(
        image: vk::Image,
        view: vk::ImageView,
    ) -> Self {
        Self {
            image,
            view,
            _phantom: PhantomData,
            _phantom2: PhantomData,
        }
    }

    // Get the internally used backing image
    pub fn image(&self) -> vk::Image {
        self.image
    }

    // Get the internally used image view
    pub fn image_view(&self) -> vk::ImageView {
        self.view
    }
}

impl<'a, T: ColorTexel> ColorAttachments<'a, T>
    for RenderTarget<'a, T>
{
    fn image_views(&self) -> Vec<vk::ImageView> {
        vec![self.image_view()]
    }
}
