use crate::{
    set_vertex_buffer_attribute,
    ActiveSceneRenderPass, DefaultMaterialResources, Material, Mesh, RenderPath, Renderer, SceneColor,
    SceneDepth, Surface, set_index_buffer_attribute,
};
use ecs::Scene;
use graphics::{RenderPipeline, ActivePipeline};
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

    // Keep track of the last attribute buffers
    let mut last_positions_buffer: Option<&<M::RenderPath as RenderPath>::AttributeBuffer<crate::attributes::Position>> = None; 
    let mut last_normals_buffer: Option<&<M::RenderPath as RenderPath>::AttributeBuffer<crate::attributes::Normal>> = None; 
    let mut last_tangents_buffer: Option<&<M::RenderPath as RenderPath>::AttributeBuffer<crate::attributes::Tangent>> = None; 
    let mut last_tex_coords_buffer: Option<&<M::RenderPath as RenderPath>::AttributeBuffer<crate::attributes::TexCoord>> = None; 
    let mut last_index_buffer: Option<&<M::RenderPath as RenderPath>::TriangleBuffer<u32>> = None;

    // Iterate over all the surface of this material
    for (surface, renderer) in query {
        // Handle non visible surfaces, renderers, and culled surfaces
        if surface.culled || !surface.visible || !renderer.visible {
            continue;
        }

        // Get the mesh and material that correspond to this surface
        let mesh = <M::RenderPath as RenderPath>::get(
            defaults,
            &surface.mesh,
        );

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
        if last_mesh != Some(surface.mesh.clone()) {
            use crate::attributes::*;
            let mut index = 0;

            // Set the position buffer attribute 
            set_vertex_buffer_attribute::<
                Position,
                M::RenderPath,
                _,
                _,
            >(
                supported, mesh, defaults, &mut active, &mut index, &mut last_positions_buffer
            );

            // Set the normal buffer attribute
            set_vertex_buffer_attribute::<Normal, M::RenderPath, _, _>(
                supported,
                mesh,
                defaults,
                &mut active,
                &mut index,
                &mut last_normals_buffer,
            );

            // Set the tangent buffer attribute
            set_vertex_buffer_attribute::<Tangent, M::RenderPath, _, _>(
                supported,
                mesh,
                defaults,
                &mut active,
                &mut index,
                &mut last_tangents_buffer
            );

            // Set the texture coordinate buffer attribute
            set_vertex_buffer_attribute::<TexCoord, M::RenderPath, _, _>(
                supported,
                mesh,
                defaults,
                &mut active,
                &mut index,
                &mut last_tex_coords_buffer
            );

            // Set the index buffer
            set_index_buffer_attribute::<M::RenderPath, _, _>(
                mesh,
                defaults,
                &mut active,
                &mut last_index_buffer,
            );

            // Set the mesh handle
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
        <M::RenderPath as RenderPath>::draw(
            mesh,
            defaults,
            &mut active,
        );

        // Add 1 to the material index when we switch instances
        if switched_material_instances {
            defaults.material_index += 1;
        }
    }
}
