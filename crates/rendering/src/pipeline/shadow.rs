use crate::{
    attributes::Position, ActiveShadowGraphicsPipeline,
    DefaultMaterialResources, Material, Mesh, MeshAttributes,
    RenderPath, Renderer, Surface, set_index_buffer_attribute, set_vertex_buffer_attribute,
};
use ecs::Scene;
use graphics::{GpuPod, ModuleVisibility, ActivePipeline};
use utils::{Handle};
use world::World;

// Render all the visible surfaces of a specific material type
pub(super) fn render_shadows<'r, M: Material>(
    world: &'r World,
    defaults: &DefaultMaterialResources<'r>,
    active: &mut ActiveShadowGraphicsPipeline<'_, 'r, '_>,
) {
    // Don't do shit if we won't cast shadows
    if !M::casts_shadows()
        || !M::attributes().contains(MeshAttributes::POSITIONS)
    {
        return;
    }

    // Get all the entities that contain a visible surface
    let scene = world.get::<Scene>().unwrap();
    let query = scene.query::<(&Surface<M>, &Renderer)>();

    // Keep track of the last model so we don't have to rebind buffers
    let mut last: Option<Handle<Mesh<M::RenderPath>>> = None;

    // Keep track of the last attribute buffers
    let mut last_positions_buffer: Option<&<M::RenderPath as RenderPath>::AttributeBuffer<crate::attributes::Position>> = None; 
    let mut last_index_buffer: Option<&<M::RenderPath as RenderPath>::TriangleBuffer<u32>> = None;

    // Iterate over all the surfaces of this material
    for (surface, renderer) in query {
        // Handle non visible surfaces, renderers, and culled surfaces
        if !surface.visible || !renderer.visible {
            continue;
        }

        // Get the mesh and material that correspond to this surface
        let mesh = <M::RenderPath as RenderPath>::get(
            defaults,
            &surface.mesh,
        );

        // Skip rendering if the mesh is invalid
        let attribute = mesh
            .vertices()
            .enabled()
            .contains(MeshAttributes::POSITIONS);
        let validity = <M::RenderPath as RenderPath>::is_valid(mesh);
        if !(attribute && validity) {
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
            // Set the position buffer attribute 
            set_vertex_buffer_attribute::<
                Position,
                M::RenderPath,
                _,
                _,
            >(
                MeshAttributes::POSITIONS, mesh, defaults, active, &mut 0, &mut last_positions_buffer
            );

            // Set the index buffer
            set_index_buffer_attribute::<M::RenderPath, _, _>(
                mesh,
                defaults,
                active,
                &mut last_index_buffer,
            );

            // Set the mesh handle
            last = Some(surface.mesh.clone());
        }

        // Draw the mesh
        <M::RenderPath as RenderPath>::draw(mesh, defaults, active);
    }
}
