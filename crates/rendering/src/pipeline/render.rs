use crate::{Material, Surface, Mesh, attributes::RawPosition};
use ecs::Scene;
use graphics::{Graphics, GraphicsPipeline, XYZ, SwapchainFormat};
use utils::{Storage, Time};
use world::World;

// Render all the visible surfaces of a specific material type
pub(super) fn render_surfaces<M: Material>(
    world: &World,
    pipeline: &GraphicsPipeline<SwapchainFormat, ()>,
    //render_pass: &mut ActiveRenderPass<SwapchainFormat, ()>
) {
    /*
    // Get a rasterizer for the current render pass by binding a pipeline
    let (mut rasterizer, mut bindings) = render_pass.bind_pipeline(pipeline);

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

        // FIXME: Figure out if we should use bindless or not
        // If we were to use bindless, how should be pass keep track of textures / buffers sequentially?

        // Bind the mesh's vertices and draw
        rasterizer.bind_vertex_buffers(&mesh.vertices().untyped_buffers());
        rasterizer.draw(mesh.vertices().len().unwrap() as u32, &bindings);
        log::debug!("Draw mesh");
    }
    */
}
