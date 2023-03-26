use crate::{
    attributes::Position, draw, ActiveShadowGraphicsPipeline,
    Material, Mesh, MeshAttributes, Renderer, Surface,
};
use ecs::Scene;
use graphics::{DrawIndexedIndirectBuffer, GpuPod, ModuleVisibility};
use utils::{Handle, Storage};
use world::World;

// Render all the visible surfaces of a specific material type
pub(super) fn render_shadows<'r, M: Material>(
    world: &'r World,
    meshes: &'r Storage<Mesh>,
    indirect: &'r Storage<DrawIndexedIndirectBuffer>,
    active: &mut ActiveShadowGraphicsPipeline<'_, 'r, '_>,
) {
    // Don't do shit if we won't cast shadows
    if !M::casts_shadows() || !M::attributes().contains(MeshAttributes::POSITIONS) {
        return;
    }

    // Get all the entities that contain a visible surface
    let scene = world.get::<Scene>().unwrap();
    let query = scene.query::<(&Surface<M>, &Renderer)>();

    // Keep track of the last model so we don't have to rebind buffers
    let mut last: Option<Handle<Mesh>> = None;

    // Iterate over all the surfaces of this material
    for (surface, renderer) in query {
        // Handle non visible surfaces, renderers, and culled surfaces
        if !surface.visible || !renderer.visible {
            continue;
        }

        // Get the mesh and material that correspond to this surface
        let mesh = meshes.get(&surface.mesh);

        // Skip rendering if the mesh is invalid
        let attribute = mesh
            .vertices()
            .enabled()
            .contains(MeshAttributes::POSITIONS);
        let validity = mesh.vertices().len().is_some();
        if !(attribute && validity) && surface.indirect.is_none() {
            continue;
        }

        // Set the mesh matrix push constant
        active
            .set_push_constants(|constants| {
                let matrix = renderer.matrix;
                let cols = matrix.cols;
                let bytes = GpuPod::into_bytes(&cols);
                constants
                    .push(bytes, 0, ModuleVisibility::Vertex)
                    .unwrap();
            })
            .unwrap();

        // Set the vertex buffers and index buffers when we change models
        if last != Some(surface.mesh.clone()) {
            // Set the position buffer
            let positions =
                mesh.vertices().attribute::<Position>().unwrap();
            active.set_vertex_buffer::<<Position as crate::MeshAttribute>::V>(0, positions, ..).unwrap();

            // Set the index buffer
            let triangles = mesh.triangles();
            active.set_index_buffer(triangles.buffer(), ..).unwrap();
            last = Some(surface.mesh.clone());
        }

        // Draw the mesh
        draw(surface, indirect, mesh, active);
    }
}
