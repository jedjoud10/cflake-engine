use std::mem::size_of;

use crate::{
    attributes::Position, ActiveShadowGraphicsPipeline,
    DefaultMaterialResources, Material, Mesh, MeshAttributes,
    RenderPath, Renderer, Surface, set_index_buffer_attribute, set_vertex_buffer_attribute,
};
use ecs::Scene;
use graphics::{GpuPod, ModuleVisibility, ActivePipeline};
use math::SharpVertices;
use utils::{Handle, ThreadPool};
use world::World;

// Check if an AABB intersects the shadow lightspace matrix
pub fn intersects_lightspace(
    lightspace: &vek::Mat4<f32>,
    aabb: math::Aabb<f32>,
    matrix: &vek::Mat4<f32>,
) -> bool {
    let corners =
        <math::Aabb<f32> as SharpVertices<f32>>::points(&aabb);

    for input in corners.iter() {
        let vec = matrix.mul_point(*input);
        let uv = lightspace.mul_point(vec);

        if uv.x.abs() < 1.0 && uv.y.abs() < 1.0 {
            return true;
        }
    }

    return false;
}

// Render all the visible surfaces of a specific material type
pub(super) fn render_shadows<'r, M: Material>(
    world: &'r World,
    defaults: &DefaultMaterialResources<'r>,
    active: &mut ActiveShadowGraphicsPipeline<'_, 'r, '_>,
    lightspace: vek::Vec4<vek::Vec4<f32>>,
) {
    // Don't do shit if we won't cast shadows
    if !M::casts_shadows()
        || !M::attributes().contains(MeshAttributes::POSITIONS)
    {
        return;
    }

    // Get all the entities that contain a visible surface
    let mut scene = world.get_mut::<Scene>().unwrap();
    let mut threadpool = world.get_mut::<ThreadPool>().unwrap();

    // Keep track of the last model so we don't have to rebind buffers
    let mut last: Option<Handle<Mesh<M::RenderPath>>> = None;

    // Keep track of the last attribute buffers
    let mut last_positions_buffer: Option<&<M::RenderPath as RenderPath>::AttributeBuffer<crate::attributes::Position>> = None; 
    let mut last_index_buffer: Option<&<M::RenderPath as RenderPath>::TriangleBuffer<u32>> = None;

    // Cull the surfaces that the shadow texture won't see
    scene.query_mut::<(&mut Surface<M>, &Renderer)>().for_each(
        &mut threadpool,
        |(surface, renderer)| {
            // Get the mesh and it's AABB
            let mesh = <M::RenderPath as RenderPath>::get(
                defaults,
                &surface.mesh,
            );
            let aabb = mesh.vertices().aabb();

            // If we have a valid AABB, check if the surface is visible within the frustum
            if let Some(aabb) = aabb {
                surface.culled = !intersects_lightspace(
                    &vek::Mat4 {
                        cols: lightspace,
                    },
                    aabb,
                    &renderer.matrix,
                )
            } else {
                surface.culled = false;
            }
        }, 
        1024,
    );

    // Iterate over all the surfaces of this material
    let query = scene.query::<(&Surface<M>, &Renderer)>();
    for (surface, renderer) in query {
        // Handle non visible surfaces, renderers, or if it's culled (shadow culled)
        if !surface.visible || !renderer.visible || surface.shadow_culled {
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
                // TODO: Implement push constant compositing so we can remove this
                let bytes = GpuPod::into_bytes(&lightspace);
                constants
                    .push(bytes, size_of::<vek::Mat4::<f32>>() as u32, ModuleVisibility::Vertex)
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
