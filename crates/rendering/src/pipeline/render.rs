use graphics::{ActiveRenderPass, GraphicsPipeline, vk, Graphics};
use world::World;
use crate::Material;

// Render all the visible surfaces of a specific material type
pub(super) fn render_surfaces<M: Material>(render_pass: &mut ActiveRenderPass, pipeline: &GraphicsPipeline) {
    render_pass.cmd_bind_pipeline(pipeline);
    /*
    let graphics = Graphics::global();
    let size = graphics.swapchain().extent();

    let viewport = vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: size.w as f32,
        height: size.h as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    };

    unsafe {
        render_pass.recorder.cmd_set_viewport(viewport);
    }
    */

    render_pass.cmd_draw(3, 1, 0, 0);
}