use crate::{PushConstants, SetPushConstantsError, BindGroup};

// Common pipeline trait that will be implemented by ActiveComputeDispatcher and ActiveGraphicsPipeline
pub trait ActivePipeline {
    // Underlying type of the pipeline
    type Pipeline;

    // Set push constants before rendering
    fn set_push_constants(
        &mut self,
        callback: impl FnOnce(&mut PushConstants<Self>),
    ) -> Result<(), SetPushConstantsError>;

    // Execute a callback that we will use to fill a bind group
    fn set_bind_group<'b>(
        &mut self,
        binding: u32,
        callback: impl FnOnce(&mut BindGroup<'b>),
    );

    // Executed before any draw call to make sure that we have
    // all the necessities (bind groups, push constants, buffers) to be able to draw
    // TODO: VALIDATION: Make sure all bind groups, push constants, and buffers, have been set
    fn validate(&self);

    // Get the underlying pipeline that was borrowed
    fn inner(&self) -> Self::Pipeline;
}