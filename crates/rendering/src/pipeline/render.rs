use crate::{
    DefaultMaterialResources, Material, Mesh, RenderPath, Renderer, SceneColorLayout, SceneDepthLayout, Surface, MeshAttribute, MeshAttributes, Pass,
};
use ecs::Scene;
use graphics::{ActivePipeline, RenderPipeline, ColorLayout, DepthStencilLayout, ActiveRenderPipeline, ActiveRenderPass};
use math::ExplicitVertices;
use utils::{Handle, Storage};
use world::World;

// Set a mesh binding vertex buffer to the current render pass
pub(crate) fn set_vertex_buffer_attribute<
    'a,
    'r,
    A: MeshAttribute,
    R: RenderPath,
    C: ColorLayout,
    DS: DepthStencilLayout,
>(
    supported: MeshAttributes,
    mesh: &'r Mesh<R>,
    defaults: &DefaultMaterialResources<'r>,
    active: &mut ActiveRenderPipeline<'a, 'r, '_, C, DS>,
    index: &mut u32,
    last: &mut Option<&'r R::AttributeBuffer<A>>,
) where
    for<'x> &'x R::AttributeBuffer<A>: PartialEq<&'x R::AttributeBuffer<A>>,
{
    // If the material doesn't support the attribute, no need to set it
    if !supported.contains(A::ATTRIBUTE) {
        return;
    }

    // Check if the mesh contains the attribute, and if it does, render it
    if let Ok(buffer) = mesh.vertices().attribute::<A>() {
        // Only set the buffer if necessary
        if *last != Some(buffer) {
            R::set_vertex_buffer(*index, .., buffer, defaults, active).unwrap();
            *last = Some(buffer);
        }

        *index += 1;
    }
}

// Set a mesh triangle buffer to the current render pass
pub(crate) fn set_index_buffer_attribute<
    'a,
    'r,
    R: RenderPath,
    C: ColorLayout,
    DS: DepthStencilLayout,
>(
    mesh: &'r Mesh<R>,
    defaults: &DefaultMaterialResources<'r>,
    active: &mut ActiveRenderPipeline<'a, 'r, '_, C, DS>,
    last: &mut Option<&R::TriangleBuffer<u32>>,
) where
    for<'x> &'x R::TriangleBuffer<u32>: PartialEq<&'x R::TriangleBuffer<u32>>,
{
    // Get the triangle buffer from the mesh
    let triangles = mesh.triangles();
    let buffer = triangles.buffer();

    // Only set the triangles if necessary
    if *last != Some(buffer) {
        R::set_index_buffer(.., buffer, defaults, active).unwrap();
    }
}


// Render all the visible surfaces of a specific material type using a specific pass
// This allows us to re-use the code for deferred pass and shadow pass albeit at a small overhead
pub(super) fn render_surfaces<'r, P: Pass, M: Material>(
    world: &'r World,
    pipeline: &'r RenderPipeline<P::C, P::DS>,
    defaults: &mut DefaultMaterialResources<'r>,
    render_pass: &mut ActiveRenderPass<'r, '_, P::C, P::DS>,
) {
    // Get a rasterizer for the current render pass by binding a pipeline
    let mut active = render_pass.bind_pipeline(pipeline);
    let supported = M::attributes::<P>();

    // Get the material storage and resources for this material
    let materials = world.get::<Storage<M>>().unwrap();
    let mut resources = M::fetch::<P>(world);

    // Set the global material bindings
    active
        .set_bind_group(0, |group| {
            M::set_global_bindings::<P>(&mut resources, group, defaults);
        })
        .unwrap();

    // Get all the entities that contain a visible surface
    let scene = world.get::<Scene>().unwrap();
    let filter = ecs::contains::<M::Query<'r>>();
    let query = scene.query_with::<(&Surface<M>, &Renderer)>(filter);

    // Get custom user components
    let filter = ecs::contains::<(&Surface<M>, &Renderer)>();
    let user = scene.query_with::<M::Query<'r>>(filter);

    // Due to the filters, these MUST have the same length
    debug_assert_eq!(query.len(), user.len());

    // Keep track of the last material
    let mut last_material: Option<Handle<M>> = None;

    // Keep track of the last model
    let mut last_mesh: Option<Handle<Mesh<M::RenderPath>>> = None;

    // Keep track of the last attribute buffers
    let mut last_positions_buffer: Option<
        &<M::RenderPath as RenderPath>::AttributeBuffer<crate::attributes::Position>,
    > = None;
    let mut last_normals_buffer: Option<
        &<M::RenderPath as RenderPath>::AttributeBuffer<crate::attributes::Normal>,
    > = None;
    let mut last_tangents_buffer: Option<
        &<M::RenderPath as RenderPath>::AttributeBuffer<crate::attributes::Tangent>,
    > = None;
    let mut last_tex_coords_buffer: Option<
        &<M::RenderPath as RenderPath>::AttributeBuffer<crate::attributes::TexCoord>,
    > = None;
    let mut last_index_buffer: Option<&<M::RenderPath as RenderPath>::TriangleBuffer<u32>> = None;

    // Convert to sub-surfaces and discard invisible / culled surfaces
    let iter = query.into_iter().zip(user);
    let iter = iter.filter(|((surface, renderer), _)| !surface.culled && surface.visible && renderer.visible);
    let subsurfaces = iter.collect::<Vec<_>>();
    let subsurfaces = subsurfaces
        .iter()
        .flat_map(|((surface, renderer), user)| 
            surface.subsurfaces.iter().map(move |x| ((x, renderer), user))
        );

    // Sort and group material instances / meshes
    // instead of [(mt1, mh1), (mt2, mh2), (mt1, mh1), (mt1, mh2)]
    // should be [(mt1, mh1), (mt1, mh1), (mt1, mh2), (mt2, mh2)]
    // Materials should have priority over meshes since they require you to set more shit
    let mut values = subsurfaces.collect::<Vec<_>>();
    values.sort_by(|((surface1, _), _), ((surface2, _), _)| {
        let mesh1 = &surface1.mesh;
        let mat1 = &surface1.material;
        let mesh2 = &surface2.mesh;
        let mat2 = &surface2.material;

        let mesh_ordering = mesh1.cmp(&mesh2);
        let material_ordering = mat1.cmp(&mat2);
        mesh_ordering.then(material_ordering)
    });

    // Iterate over all the surface of this material
    for ((subsurface, renderer), user) in values {
        // Get the mesh and material that correspond to this surface
        let mesh = <M::RenderPath as RenderPath>::get(defaults, &subsurface.mesh);

        // Check if we changed material instances
        if last_material != Some(subsurface.material.clone()) {
            last_material = Some(subsurface.material.clone());
            let material = materials.get(&subsurface.material);

            // Set the instance group bindings
            active
                .set_bind_group(1, |group| {
                    M::set_instance_bindings::<P>(material, &mut resources, defaults, group);
                })
                .unwrap();
        }

        // If a mesh is missing attributes just skip
        if !mesh.vertices().enabled().contains(supported) {
            continue;
        }

        // If a mesh isn't valid we have a problem, not so big but still a problem
        if !<M::RenderPath as RenderPath>::is_valid(defaults, mesh) {
            log::warn!("Mesh invalid! Check buffers or indexed indirect count/offset (normal render pipe)");
            continue;
        }

        // Set the surface group bindings
        active
            .set_bind_group(2, |group| {
                M::set_surface_bindings::<P>(renderer, &mut resources, defaults, &user, group);
            })
            .unwrap();

        // Set the vertex buffers and index buffers when we change meshes
        if last_mesh != Some(subsurface.mesh.clone()) {
            use crate::attributes::*;
            let mut index = 0;

            // Set the position buffer attribute
            set_vertex_buffer_attribute::<Position, M::RenderPath, _, _>(
                supported,
                mesh,
                defaults,
                &mut active,
                &mut index,
                &mut last_positions_buffer,
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
                &mut last_tangents_buffer,
            );

            // Set the texture coordinate buffer attribute
            set_vertex_buffer_attribute::<TexCoord, M::RenderPath, _, _>(
                supported,
                mesh,
                defaults,
                &mut active,
                &mut index,
                &mut last_tex_coords_buffer,
            );

            // Set the index buffer
            set_index_buffer_attribute::<M::RenderPath, _, _>(
                mesh,
                defaults,
                &mut active,
                &mut last_index_buffer,
            );

            // Set the mesh handle
            last_mesh = Some(subsurface.mesh.clone());
        }

        // Set the push constant ranges right before rendering (in the hot loop!)
        active
            .set_push_constants(|push_constants| {
                let material = materials.get(&subsurface.material);
                M::set_push_constants::<P>(
                    material,
                    renderer,
                    &mut resources,
                    defaults,
                    &user,
                    push_constants,
                );
            })
            .unwrap();

        // Draw the mesh
        <M::RenderPath as RenderPath>::draw(mesh, defaults, &mut active).unwrap();
    }
}
