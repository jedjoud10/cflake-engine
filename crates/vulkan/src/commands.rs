use super::Recorder;
use ahash::AHashMap;
use ash::vk;


/*
    vkCmdBeginConditionalRenderingEXT(3)
vkCmdBeginDebugUtilsLabelEXT(3)
vkCmdBeginQuery(3)
vkCmdBeginQueryIndexedEXT(3)
vkCmdBeginRenderPass(3)
vkCmdBeginRenderPass2(3)
vkCmdBeginTransformFeedbackEXT(3)
vkCmdBindDescriptorSets(3)
vkCmdBindIndexBuffer(3)
vkCmdBindPipeline(3)
vkCmdBindShadingRateImageNV(3)
vkCmdBindTransformFeedbackBuffersEXT(3)
vkCmdBindVertexBuffers(3)
vkCmdBlitImage(3)
vkCmdBuildAccelerationStructureNV(3)
vkCmdClearAttachments(3)
vkCmdClearColorImage(3)
vkCmdClearDepthStencilImage(3)
vkCmdCopyAccelerationStructureNV(3)
vkCmdCopyBuffer(3)
vkCmdCopyBufferToImage(3)
vkCmdCopyImage(3)
vkCmdCopyImageToBuffer(3)
vkCmdCopyQueryPoolResults(3)
vkCmdDebugMarkerBeginEXT(3)
vkCmdDebugMarkerEndEXT(3)
vkCmdDebugMarkerInsertEXT(3)
vkCmdDispatch(3)
vkCmdDispatchBase(3)
vkCmdDispatchIndirect(3)
vkCmdDraw(3)
vkCmdDrawIndexed(3)
vkCmdDrawIndexedIndirect(3)
vkCmdDrawIndexedIndirectCount(3)
vkCmdDrawIndirect(3)
vkCmdDrawIndirectByteCountEXT(3)
vkCmdDrawIndirectCount(3)
vkCmdDrawMeshTasksIndirectCountNV(3)
vkCmdDrawMeshTasksIndirectNV(3)
vkCmdDrawMeshTasksNV(3)
vkCmdEndConditionalRenderingEXT(3)
vkCmdEndDebugUtilsLabelEXT(3)
vkCmdEndQuery(3)
vkCmdEndQueryIndexedEXT(3)
vkCmdEndRenderPass(3)
vkCmdEndRenderPass2(3)
vkCmdEndTransformFeedbackEXT(3)
vkCmdExecuteCommands(3)
vkCmdFillBuffer(3)
vkCmdInsertDebugUtilsLabelEXT(3)
vkCmdNextSubpass(3)
vkCmdNextSubpass2(3)
vkCmdPipelineBarrier(3)
vkCmdProcessCommandsNVX(3)
vkCmdPushConstants(3)
vkCmdPushDescriptorSetKHR(3)
vkCmdPushDescriptorSetWithTemplateKHR(3)
vkCmdReserveSpaceForCommandsNVX(3)
vkCmdResetEvent(3)
vkCmdResetQueryPool(3)
vkCmdResolveImage(3)
vkCmdSetBlendConstants(3)
vkCmdSetCheckpointNV(3)
vkCmdSetCoarseSampleOrderNV(3)
vkCmdSetDepthBias(3)
vkCmdSetDepthBounds(3)
vkCmdSetDeviceMask(3)
vkCmdSetDiscardRectangleEXT(3)
vkCmdSetEvent(3)
vkCmdSetExclusiveScissorNV(3)
vkCmdSetLineStippleEXT(3)
vkCmdSetLineWidth(3)
vkCmdSetPerformanceMarkerINTEL(3)
vkCmdSetPerformanceOverrideINTEL(3)
vkCmdSetPerformanceStreamMarkerINTEL(3)
vkCmdSetSampleLocationsEXT(3)
vkCmdSetScissor(3)
vkCmdSetStencilCompareMask(3)
vkCmdSetStencilReference(3)
vkCmdSetStencilWriteMask(3)
vkCmdSetViewport(3)
vkCmdSetViewportShadingRatePaletteNV(3)
vkCmdSetViewportWScalingNV(3)
vkCmdTraceRaysNV(3)
vkCmdUpdateBuffer(3)
vkCmdWaitEvents(3)
vkCmdWriteAccelerationStructuresPropertiesNV(3)
vkCmdWriteBufferMarkerAMD(3)
vkCmdWriteTimestamp(3)

*/

// Buffer commands
impl Recorder {
    // Bind an index buffer to the command buffer render pass
    pub unsafe fn cmd_bind_index_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        index_type: vk::IndexType,
    ) {
    }

    // Bind vertex buffers to the command buffer render pass
    pub unsafe fn cmd_bind_vertex_buffers(
        &mut self,
        first_binding: u32,
        buffers: Vec<vk::Buffer>,
        offsets: Vec<vk::DeviceSize>,
    ) {
    }

    // Copy a buffer to another buffer in GPU memory
    pub unsafe fn cmd_copy_buffer(
        &mut self,
        src: vk::Buffer,
        dst: vk::Buffer,
        regions: Vec<vk::BufferCopy>,
    ) {
    }

    // Copy an image to a buffer in GPU memory
    pub unsafe fn cmd_copy_image_to_buffer(
        &mut self,
        buffer: vk::Buffer,
        image: vk::Image,
        layout: vk::ImageLayout,
        regions: Vec<vk::BufferImageCopy>,
    ) {
    }

    // Clear a buffer to zero
    pub unsafe fn cmd_clear_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
    ) {
    }

    // Update the buffer using memory that is directly stored within the command buffer
    pub unsafe fn cmd_update_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        data: Vec<u8>,
    ) {
    }
}

// Image commands
impl Recorder {
    // Blit an image to another image in GPU memory
    pub unsafe fn cmd_blit_image(
        &mut self,
        src_image: vk::Image,
        src_layout: vk::ImageLayout,
        dst_image: vk::Image,
        dst_layout: vk::ImageLayout,
        regions: Vec<vk::ImageBlit>,
        filter: vk::Filter,
    ) {
    }

    // Clear an image to a specific color 
    pub unsafe fn cmd_clear_image(
        &mut self,
        image: vk::Image,
        layout: vk::ImageLayout,
        color: vk::ClearColorValue,
        regions: Vec<vk::ImageSubresourceRange>,
    ) {
    }

    // Copy an image to another image in GPU memory
    pub unsafe fn cmd_copy_image(
        &mut self,
        src_image: vk::Image,
        src_layout: vk::ImageLayout,
        dst_image: vk::Image,
        dst_layout: vk::ImageLayout,
        regions: Vec<vk::ImageCopy>,
    ) {
    }
}