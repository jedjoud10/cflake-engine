use crate::{Material, Surface, Mesh, attributes::{RawPosition, Position}, MeshAttribute};
use ecs::Scene;
use graphics::{Graphics, GraphicsPipeline, XYZ, SwapchainFormat, ActiveRenderPass, Vertex};
use utils::{Storage, Time};
use world::World;

// Set a mesh binding vertex buffer to the current render pass
pub(crate) fn set_vertex_buffer_attribute<'r, M: MeshAttribute>(
    mesh: &'r Mesh,
    render_pass: &mut ActiveRenderPass<'r, '_, '_, SwapchainFormat, ()>
) {
    if let Some(buffer) = mesh.vertices().attribute::<M>() {
        render_pass.set_vertex_buffer::<M::V>(M::index(), buffer);
    }
}

// Render all the visible surfaces of a specific material type
pub(super) fn render_surfaces<'r, M: Material>(
    world: &'r World,
    meshes: &'r Storage<Mesh>,
    pipeline: &'r GraphicsPipeline<SwapchainFormat, ()>,
    render_pass: &mut ActiveRenderPass<'r, '_, '_, SwapchainFormat, ()>
) {
    // Get a rasterizer for the current render pass by binding a pipeline
    render_pass.bind_pipeline(pipeline);

    // Get all the meshes and surface for this specific material
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
        use crate::attributes::*;
        set_vertex_buffer_attribute::<Position>(mesh, render_pass);
        set_vertex_buffer_attribute::<Normal>(mesh, render_pass);
        set_vertex_buffer_attribute::<Tangent>(mesh, render_pass);
        set_vertex_buffer_attribute::<TexCoord>(mesh, render_pass);

        // Set the index buffer
        let triangles = mesh.triangles();
        render_pass.set_index_buffer(triangles.buffer());

        // Draw the triangulated mesh 
        let indices = 0..(triangles.buffer().len() as u32);
        render_pass.draw_indexed(indices, 0..1);
    }
}
