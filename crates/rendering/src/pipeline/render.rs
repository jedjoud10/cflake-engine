use crate::{Material, SwapchainFormat};
use graphics::{vk, Graphics, GraphicsPipeline, Rasterizer};
use world::World;

// Render all the visible surfaces of a specific material type
pub(super) fn render_surfaces<M: Material>(
    world: &World,
    pipeline: &GraphicsPipeline,
    rasterizer: &mut Rasterizer<'_, '_, '_, SwapchainFormat, ()>
) {
    rasterizer.bind_pipeline(pipeline);
    rasterizer.draw(6, 1, 0, 0);
}
