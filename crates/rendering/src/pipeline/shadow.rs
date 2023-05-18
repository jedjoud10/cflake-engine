use std::mem::size_of;

use crate::{
    attributes::Position, set_index_buffer_attribute, set_vertex_buffer_attribute,
    ActiveShadowRenderPass, ActiveShadowRenderPipeline, CastShadowsMode, DefaultMaterialResources,
    Material, Mesh, MeshAttributes, RenderPath, Renderer, ShadowRenderPipeline, SubSurface,
    Surface,
};
use ecs::Scene;
use graphics::{ActivePipeline, GpuPod, ModuleVisibility};
use math::ExplicitVertices;
use utils::{Handle};
use world::World;

// Check if an AABB intersects the shadow lightspace matrix
pub fn intersects_lightspace(
    lightspace: &vek::Mat4<f32>,
    aabb: math::Aabb<f32>,
    matrix: &vek::Mat4<f32>,
) -> bool {
    let corners = <math::Aabb<f32> as ExplicitVertices<f32>>::points(&aabb);

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
    render_pass: &mut ActiveShadowRenderPass<'r, '_>,
    shadow_pipeline: &'r ShadowRenderPipeline,
    lightspace: vek::Mat4<f32>,
) {
    let mut active = render_pass.bind_pipeline(shadow_pipeline);

    // Don't do shit if we won't cast shadows
    if matches!(M::casts_shadows(), CastShadowsMode::Disabled)
        || !M::attributes().contains(MeshAttributes::POSITIONS)
    {
        return;
    }

    // Get all the entities that contain a visible surface
    let mut scene = world.get_mut::<Scene>().unwrap();

    // Keep track of the last model so we don't have to rebind buffers
    let mut last: Option<Handle<Mesh<M::RenderPath>>> = None;

    // Keep track of the last attribute buffers
    let mut last_positions_buffer: Option<
        &<M::RenderPath as RenderPath>::AttributeBuffer<crate::attributes::Position>,
    > = None;
    let mut last_index_buffer: Option<&<M::RenderPath as RenderPath>::TriangleBuffer<u32>> = None;

    // Cull the surfaces that the shadow texture won't see
    /*
    if M::frustum_culling() {
        scene.query_mut::<(&mut Surface<M>, &Renderer)>().for_each(
            &mut threadpool,
            |(surface, renderer)| {
                // A surface is culled *only* if all of it's sub-surface are not visible
                surface.shadow_culled =
                    surface
                        .subsurfaces
                        .iter()
                        .all(|SubSurface { mesh, material }| {
                            // Get the mesh and it's AABB
                            let mesh = <M::RenderPath as RenderPath>::get(defaults, &mesh);
                            let aabb = mesh.vertices().aabb();

                            // If we have a valid AABB, check if the surface is visible within the frustum
                            if let Some(aabb) = aabb {
                                !intersects_lightspace(&lightspace, aabb, &renderer.matrix)
                            } else {
                                false
                            }
                        })
            },
            shadow_frustum_culling_batch_size,
        );
    }
    */

    // Iterate over all the surfaces of this material
    let query = scene.query::<(&Surface<M>, &Renderer)>();
    for (surface, renderer) in query {
        // Handle non visible surfaces, renderers, or if it's culled (shadow culled)
        if !surface.visible || !renderer.visible || surface.shadow_culled {
            continue;
        }

        // Iterate over the sub-surfaces of the surface
        for subsurface in surface.subsurfaces.iter() {
            // Get the mesh and material that correspond to this surface
            let mesh = <M::RenderPath as RenderPath>::get(defaults, &subsurface.mesh);

            // If a mesh is missing attributes just skip
            if !mesh.vertices().enabled().contains(MeshAttributes::POSITIONS) {
                continue;
            }

            // If a mesh isn't valid we have a problem, not so big but still a problem
            if !<M::RenderPath as RenderPath>::is_valid(&defaults, mesh) {
                log::warn!("Mesh invalid! Check buffers or indexed indirect count/offset (shadow render pipe)");
                continue;
            }

            // Set the mesh matrix push constant
            active
                .set_push_constants(|constants| {
                    let matrix = renderer.matrix;
                    let cols = matrix.cols;
                    let bytes = GpuPod::into_bytes(&cols);
                    constants.push(bytes, 0, ModuleVisibility::Vertex).unwrap();
                    // TODO: Implement push constant compositing so we can remove this
                    let bytes = GpuPod::into_bytes(&lightspace.cols);
                    constants
                        .push(
                            bytes,
                            size_of::<vek::Mat4<f32>>() as u32,
                            ModuleVisibility::Vertex,
                        )
                        .unwrap();
                })
                .unwrap();

            // Set the vertex buffers and index buffers when we change models
            if last != Some(subsurface.mesh.clone()) {
                // Set the position buffer attribute
                set_vertex_buffer_attribute::<Position, M::RenderPath, _, _>(
                    MeshAttributes::POSITIONS,
                    mesh,
                    defaults,
                    &mut active,
                    &mut 0,
                    &mut last_positions_buffer,
                );

                // Set the index buffer
                set_index_buffer_attribute::<M::RenderPath, _, _>(
                    mesh,
                    defaults,
                    &mut active,
                    &mut last_index_buffer,
                );

                // Set the mesh handle
                last = Some(subsurface.mesh.clone());
            }

            // Draw the mesh
            <M::RenderPath as RenderPath>::draw(mesh, defaults, &mut active).unwrap();
        }
    }
}
