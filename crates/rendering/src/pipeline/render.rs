use crate::{
    ActiveScenePipeline, ActiveSceneRenderPass,
    DefaultMaterialResources, Material, Mesh, MeshAttribute,
    MeshAttributes, Renderer, SceneColor, SceneDepth, Surface, RenderPath, set_vertex_buffer_attribute,
};
use ecs::Scene;
use graphics::{DrawIndexedIndirectBuffer, RenderPipeline};
use utils::{Handle, Storage};
use world::World;

// Render all the visible surfaces of a specific material type
pub(super) fn render_surfaces<'r, M: Material>(
    world: &'r World,
    pipeline: &'r RenderPipeline<SceneColor, SceneDepth>,
    defaults: &mut DefaultMaterialResources<'r>,
    render_pass: &mut ActiveSceneRenderPass<'r, '_>,
) {
    // Reset the material resources for this new material type
    defaults.material_index = 0;
    defaults.draw_call_index = 0;

    // Get a rasterizer for the current render pass by binding a pipeline
    let mut active = render_pass.bind_pipeline(pipeline);
    let supported = M::attributes();

    // Get the material storage and resources for this material
    let materials = world.get::<Storage<M>>().unwrap();
    let mut resources = M::fetch(world);

    // Set the global material bindings
    active.set_bind_group(0, |group| {
        M::set_global_bindings(&mut resources, group, defaults);
    });

    // Get all the entities that contain a visible surface
    let scene = world.get::<Scene>().unwrap();
    let query = scene.query::<(&Surface<M>, &Renderer)>();

    // Keep track of the last material
    let mut last_material: Option<Handle<M>> = None;
    let mut switched_material_instances;

    // Keep track of the last model
    let mut last_mesh: Option<Handle<Mesh<M::RenderPath>>> = None;

    // Iterate over all the surface of this material
    for (surface, renderer) in query {
        // Handle non visible surfaces, renderers, and culled surfaces
        if surface.culled || !surface.visible || !renderer.visible {
            continue;
        }

        // Get the mesh and material that correspond to this surface
        let mesh = <M::RenderPath as RenderPath>::get(&defaults, &surface.mesh);

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
                    defaults,
                    group,
                );
            })
        } else {
            switched_material_instances = false;
        }

        // Skip rendering if the mesh is invalid
        let attribute =
            mesh.vertices().enabled().contains(M::attributes());
        let validity = <M::RenderPath as RenderPath>::is_valid(mesh);
        if !(attribute && validity) {
            continue;
        }

        // Set the surface group bindings
        active.set_bind_group(2, |group| {
            M::set_surface_bindings(
                renderer,
                &mut resources,
                defaults,
                group,
            );
        });

        // Set the vertex buffers and index buffers when we change meshes
        // TODO: Optimize this further by not setting the same vertex buffer twice in case of indirectly drawn meshes (shared vetex buffer handles)
        if last_mesh != Some(surface.mesh.clone()) {
            use crate::attributes::*;
            let mut index = 0;
            set_vertex_buffer_attribute::<Position, M::RenderPath, _, _>(
                supported,
                mesh,
                defaults,
                &mut active,
                &mut index,
            );
            set_vertex_buffer_attribute::<Normal, M::RenderPath, _, _>(
                supported,
                mesh,
                defaults,
                &mut active,
                &mut index,
            );
            set_vertex_buffer_attribute::<Tangent, M::RenderPath, _, _>(
                supported,
                mesh,
                defaults,
                &mut active,
                &mut index,
            );
            set_vertex_buffer_attribute::<TexCoord, M::RenderPath, _, _>(
                supported,
                mesh,
                defaults,
                &mut active,
                &mut index,
            );

            // Set the index buffer
            let triangles = mesh.triangles();
            <M::RenderPath as RenderPath>::set_index_buffer(.., triangles.buffer(), defaults, &mut active).unwrap();
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
                    defaults,
                    push_constants,
                );
            })
            .unwrap();

        // Draw the mesh
        <M::RenderPath as RenderPath>::draw(mesh, &defaults, &mut active);

        // Add 1 to the material index when we switch instances
        if switched_material_instances {
            defaults.material_index += 1;
        }
    }
}
