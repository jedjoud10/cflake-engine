use vulkan::vk;

use crate::{Texel, UntypedTexel, LoadOp, StoreOp, Stencil};

// A render pass attachment is like a render texture that we write to 
// whenever we render something in the render pass
pub struct Attachment<T: Texel> {
    format: vk::Format,
    sample_count: vk::SampleCountFlags,
    load_op: LoadOp<T>,
    store_op: StoreOp,
    stencil_load_op: LoadOp<Stencil<u8>>,
    stencil_store_op: StoreOp,
    initial_layout: vk::ImageLayout,
    final_layout: vk::ImageLayout,
}

// Dynamic attachment without the texel (basically a render target trait)
pub trait DynamicAttachment {
    // Get the untyped texel format
    fn untyped(&self) -> UntypedTexel;

    // Get the underlying image that must be used for rendering
    fn image(&self) -> vk::Image;

    // Get the underlying image view that must be used for rendering
    fn image_view(&self) -> vk::ImageView;
}