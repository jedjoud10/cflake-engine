use crate::{
    ActiveScenePipeline, ActiveSceneRenderPass,
    DefaultMaterialResources, MeshAttributes, Material, Mesh,
    MeshAttribute, Renderer, SceneColor, SceneDepth, Surface,
};
use ecs::Scene;
use graphics::{RenderPipeline, DrawIndexedIndirectBuffer};
use utils::{Handle, Storage};
use world::World;

use super::draw;

// Set a mesh binding vertex buffer to the current render pass
pub(crate) fn set_vertex_buffer_attribute<
    'a,
    'r,
    A: MeshAttribute,
>(
    supported: MeshAttributes,
    mesh: &'r Mesh,
    active: &mut ActiveScenePipeline<'a, 'r, '_>,
) {
    // If the material doesn't support the attribute, no need to set it
    if !supported.contains(A::ATTRIBUTE) {
        return;
    }

    // Check if the mesh contains the attribute, and if it does, render it
    if let Some(buffer) = mesh.vertices().attribute::<A>() {
        active
            .set_vertex_buffer::<A::V>(A::index(), buffer, ..)
            .unwrap();
    }
}

// Render all the visible surfaces of a specific material type
pub(super) fn render_surfaces<'r, M: Material>(
    world: &'r World,
    meshes: &'r Storage<Mesh>,
    indirect: &'r Storage<DrawIndexedIndirectBuffer>,
    pipeline: &'r RenderPipeline<SceneColor, SceneDepth>,
    default: &mut DefaultMaterialResources<'r>,
    render_pass: &mut ActiveSceneRenderPass<'r, '_>,
) {
    // Reset the material resources for this new material type
    default.material_index = 0;
    default.draw_call_index = 0;

    // Get a rasterizer for the current render pass by binding a pipeline
    let mut active = render_pass.bind_pipeline(pipeline);
    let supported = M::attributes();

    // Get the material storage and resources for this material
    let materials = world.get::<Storage<M>>().unwrap();
    let mut resources = M::fetch(world);

    // Set the global material bindings
    active.set_bind_group(0, |group| {
        M::set_global_bindings(&mut resources, group, default);
    });

    // Get all the entities that contain a visible surface
    let scene = world.get::<Scene>().unwrap();
    let query = scene.query::<(&Surface<M>, &Renderer)>();

    // Keep track of the last material
    let mut last_material: Option<Handle<M>> = None;
    let mut switched_material_instances;

    // Keep track of the last model
    let mut last_mesh: Option<Handle<Mesh>> = None;

    // Iterate over all the surface of this material
    for (surface, renderer) in query {
        // Handle non visible surfaces, renderers, and culled surfaces
        if surface.culled || !surface.visible || !renderer.visible {
            continue;
        }

        // Get the mesh and material that correspond to this surface
        let mesh = meshes.get(&surface.mesh);

        // Check if we changed material instances
        if last_material != Some(surface.material.clone()) {
            switched_material_instances = true;
            last_material = Some(surface.material.clone());
            let material = materials.get(&surface.material);

            // Set the instance group bindings
            active.set_bind_group(1, |group| {
                M::set_instance_bindings(
                    material,
                    &mut resources,
                    default,
                    group,
                );
            })
        } else {
            switched_material_instances = false;
        }

        // Skip rendering if the mesh is invalid
        let attribute =
            mesh.vertices().enabled().contains(M::attributes());
        let validity = mesh.vertices().len().is_some();
        if !(attribute && validity) && surface.indirect.is_none() {
            continue;
        }

        // Set the surface group bindings
        active.set_bind_group(2, |group| {
            M::set_surface_bindings(
                renderer,
                &mut resources,
                default,
                group,
            );
        });

        // Set the vertex buffers and index buffers when we change meshes
        if last_mesh != Some(surface.mesh.clone()) {
            use crate::attributes::*;
            set_vertex_buffer_attribute::<Position>(
                supported,
                mesh,
                &mut active,
            );
            set_vertex_buffer_attribute::<Normal>(
                supported,
                mesh,
                &mut active,
            );
            set_vertex_buffer_attribute::<Tangent>(
                supported,
                mesh,
                &mut active,
            );
            set_vertex_buffer_attribute::<TexCoord>(
                supported,
                mesh,
                &mut active,
            );

            // Set the index buffer
            let triangles = mesh.triangles();
            active.set_index_buffer(triangles.buffer(), ..).unwrap();
            last_mesh = Some(surface.mesh.clone());
        }

        // Set the push constant ranges right before rendering (in the hot loop!)
        active
            .set_push_constants(|push_constants| {
                let material = materials.get(&surface.material);
                M::set_push_constants(
                    material,
                    renderer,
                    &mut resources,
                    default,
                    push_constants,
                );
            })
            .unwrap();

        // Draw the mesh
        draw(surface, indirect, mesh, &mut active);

        // Add 1 to the material index when we switch instances
        if switched_material_instances {
            default.material_index += 1;
        }
    }
}
