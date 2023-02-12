use crate::{Material, Surface, Mesh, attributes::RawPosition};
use ecs::Scene;
use graphics::{Graphics, GraphicsPipeline, XYZ, SwapchainFormat, ActiveRenderPass};
use utils::{Storage, Time};
use world::World;

// Render all the visible surfaces of a specific material type
pub(super) fn render_surfaces<'r, M: Material>(
    world: &World,
    pipeline: &'r GraphicsPipeline<SwapchainFormat, ()>,
    render_pass: &mut ActiveRenderPass<'r, '_, '_, SwapchainFormat, ()>
) {
    // Get a rasterizer for the current render pass by binding a pipeline
    render_pass.bind_pipeline(pipeline);
    render_pass.draw(0..6, 0..1);

    /*
    // Get all the meshes and surface for this specific material
    let meshes = world.get::<Storage<Mesh>>().unwrap();
    let materials = world.get::<Storage<M>>().unwrap();

    // Get all the entities that contain a visible surface
    let scene = world.get::<Scene>().unwrap();
    let query = scene.query::<&Surface<M>>();

    // Iterate over all the surface of this material
    for surface in query {
        // Get the mesh and material that correspond to this surface
        let mesh = meshes.get(&surface.mesh);
        let material = materials.get(&surface.material);

        // Bind the mesh's vertex buffers
        //render_pass.set_vertex_buffer(0, mesh.p);

        // Draw the mesh
        // TODO: Use indirect drawing isntead for ze performance
        render_pass.draw(0..6, 0..1);
    }
    */
}
