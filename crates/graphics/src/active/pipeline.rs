use std::error::Error;
use super::{PushConstants, SetPushConstantsError, SetBindGroupError, BindGroup};

// Common pipeline trait that will be implemented by ActiveComputeDispatcher and ActiveRenderPipeline
pub trait ActivePipeline {
    // Underlying type of the pipeline
    type Pipeline;

    // Set the push constants for the active pipeline dynamically
    fn set_push_constants(
        &mut self,
        callback: impl FnOnce(&mut PushConstants<Self>),
    ) -> Result<(), SetPushConstantsError>;

    // Set the bind group at the specific binding ID
    fn set_bind_group<'b>(
        &mut self,
        binding: u32,
        bind_group: BindGroup,
    ) -> Result<(), SetBindGroupError>;

    // Get the underlying pipeline that was borrowed
    fn inner(&self) -> Self::Pipeline;
}
