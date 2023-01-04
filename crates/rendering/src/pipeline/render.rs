use crate::Material;
use graphics::{vk, Graphics, GraphicsPipeline};
use world::World;

// Render all the visible surfaces of a specific material type
pub(super) fn render_surfaces<M: Material>(
    pipeline: &GraphicsPipeline,
) {
}
