use std::error::Error;

use crate::{PushConstants, SetPushConstantsError, BindGroup, SetBindGroupError};

// Common pipeline trait that will be implemented by ActiveComputeDispatcher and ActiveGraphicsPipeline
pub trait ActivePipeline {
    // Underlying type of the pipeline
    type Pipeline;

    // Set push constants before rendering
    // TODO: Currently, push constants can be "composed" by calling this method multiple times with different offsets
    // I gotta implement the same mechanic for bind groups as well in the future 
    fn set_push_constants(
        &mut self,
        callback: impl FnOnce(&mut PushConstants<Self>),
    ) -> Result<(), SetPushConstantsError>;

    // Execute a callback that we will use to fill a bind group
    // Might fail if the user forgets to set a bind resource or if the index is too high
    fn set_bind_group<'b>(
        &mut self,
        binding: u32,
        callback: impl FnOnce(&mut BindGroup<'b>),
    ) -> Result<(), SetBindGroupError>;

    // Get the underlying pipeline that was borrowed
    fn inner(&self) -> Self::Pipeline;
}