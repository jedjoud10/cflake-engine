use crate::{Material, SwapchainFormat, RenderSurface, Mesh, attributes::RawPosition};
use ecs::Scene;
use graphics::{vk, Graphics, GraphicsPipeline, Rasterizer, XYZ};
use utils::Storage;
use world::World;

// Render all the visible surfaces of a specific material type
pub(super) fn render_surfaces<M: Material>(
    world: &World,
    pipeline: &GraphicsPipeline,
    rasterizer: &mut Rasterizer<'_, '_, '_, SwapchainFormat, ()>
) {
    rasterizer.bind_pipeline(pipeline);
    rasterizer.draw(6, 1, 0, 0);

    /*
    let scene = world.get::<Scene>().unwrap();
    let meshes = world.get::<Storage<Mesh>>().unwrap();
    let materials = world.get::<Storage<M>>().unwrap();
    let query = scene.query::<&RenderSurface<M>>(); 

    for surface in query {
        let mesh = meshes.get(&surface.mesh);
    }
    */
}
