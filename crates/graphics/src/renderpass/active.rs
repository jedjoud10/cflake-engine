// This is an active render pass that is currently
// able to render to the framebuffer attachments
/*
pub struct ActiveRenderPass<'a, 'r> {
    pub(super) renderpass: &'a RenderPass,
    pub recorder: Recorder<'r>,
}

impl<'a, 'r> ActiveRenderPass<'a, 'r> {
    pub fn cmd_bind_pipeline(
        &mut self,
        pipeline: &GraphicsPipeline,
    ) {
        unsafe {
            self.recorder.cmd_bind_pipeline(pipeline.raw(), vk::PipelineBindPoint::GRAPHICS);
        }
    }

    pub fn cmd_draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32
    ) {
        unsafe {
            self.recorder.cmd_draw(vertex_count, instance_count, first_vertex, first_instance);
        }
    }

    pub fn set_viewport()

    pub unsafe fn end(mut self) {
        log::debug!("End active render pass");
        self.recorder.cmd_end_render_pass();
        self.recorder.immediate_submit();
    }
}
*/

/*
impl<'a, 'r> Drop for ActiveRenderPass<'a, 'r> {
    fn drop(&mut self) {
        unsafe {

        }
    }
}
*/
