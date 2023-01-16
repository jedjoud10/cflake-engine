use crate::{Material, SwapchainFormat, RenderSurface, Mesh, attributes::RawPosition};
use ecs::Scene;
use graphics::{vk, Graphics, GraphicsPipeline, ActiveRenderPass, XYZ};
use utils::{Storage, Time};
use world::World;

// Render all the visible surfaces of a specific material type
pub(super) fn render_surfaces<M: Material>(
    world: &World,
    pipeline: &GraphicsPipeline,
    rasterizer: &mut ActiveRenderPass<'_, '_, '_, SwapchainFormat, ()>
) {
    let time = world.get::<Time>().unwrap();
    
    let (mut active, mut uniforms ) = rasterizer.bind_pipeline(pipeline);
    uniforms.set_push_constants::<u32>("mesh_data", &2);
    active.draw(6, &uniforms);
    
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
