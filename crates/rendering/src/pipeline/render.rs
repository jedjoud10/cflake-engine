use crate::{Material, SwapchainFormat, RenderSurface, Mesh, attributes::RawPosition};
use ecs::Scene;
use graphics::{vk, Graphics, GraphicsPipeline, ActiveRenderPass, XYZ, ModuleKind};
use utils::{Storage, Time};
use world::World;

// Render all the visible surfaces of a specific material type
pub(super) fn render_surfaces<M: Material>(
    world: &World,
    pipeline: &GraphicsPipeline,
    rasterizer: &mut ActiveRenderPass<'_, '_, '_, SwapchainFormat, ()>
) {
    let time = world.get::<Time>().unwrap();
    
    let (mut active, mut bindings) = rasterizer.bind_pipeline(pipeline);
    
    //bindings.set_push_constant::<u32>("mesh_data", "value", &2).unwrap();
    bindings.push_constant
    active.draw(6);
    active.draw(6);
    
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
