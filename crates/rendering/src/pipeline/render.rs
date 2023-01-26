use crate::{Material, SwapchainFormat, RenderSurface, Mesh, attributes::RawPosition};
use ecs::Scene;
use graphics::{vk, Graphics, GraphicsPipeline, ActiveRenderPass, XYZ, ModuleKind};
use utils::{Storage, Time};
use world::World;

// Render all the visible surfaces of a specific material type
pub(super) fn render_surfaces<M: Material>(
    world: &World,
    pipeline: &GraphicsPipeline,
    render_pass: &mut ActiveRenderPass<SwapchainFormat, ()>
) {
    let time = world.get::<Time>().unwrap();
    
    // Get a rasterizer for the current render pass by binding a pipeline
    let (mut rasterizer, mut bindings) = render_pass.bind_pipeline(pipeline);

    //bindings.set_block::<u32>("mesh_data", &2).unwrap();
    rasterizer.draw(6, &bindings);
    rasterizer.draw(6, &bindings);
    
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
