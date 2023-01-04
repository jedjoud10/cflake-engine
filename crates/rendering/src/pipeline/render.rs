use crate::{Material, SwapchainFormat};
use graphics::{vk, Graphics, GraphicsPipeline, Rasterizer};
use world::World;

// Render all the visible surfaces of a specific material type
pub(super) fn render_surfaces<M: Material>(
    pipeline: &GraphicsPipeline,
    rasterizer: &mut Rasterizer<'_, '_, '_, SwapchainFormat, ()>
) {
    rasterizer.cmd_bind_pipeline(pipeline);
    rasterizer.cmd_draw(3, 1, 0, 0);
}
