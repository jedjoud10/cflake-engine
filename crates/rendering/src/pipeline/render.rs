use crate::{Material, Surface, Mesh, attributes::{RawPosition, Position}, MeshAttribute, EnabledMeshAttributes, Renderer, DefaultMaterialResources};
use ecs::Scene;
use graphics::{Graphics, GraphicsPipeline, XYZ, SwapchainFormat, ActiveRenderPass, Vertex, ActiveGraphicsPipeline};
use utils::{Storage, Time};
use world::World;

// Set a mesh binding vertex buffer to the current render pass
pub(crate) fn set_vertex_buffer_attribute<'a, 'r, A: MeshAttribute>(
    supported: EnabledMeshAttributes,
    mesh: &'r Mesh,
    active: &mut ActiveGraphicsPipeline<'a, 'r, '_, '_, SwapchainFormat, ()>
) {
    // If the material doesn't support the attribute, no need to set it
    if !supported.contains(A::ATTRIBUTE) {
        return;
    }

    // Check if the mesh contains the attribute, and if it does, render it
    if let Some(buffer) = mesh.vertices().attribute::<A>() {
        active.set_vertex_buffer::<A::V>(A::index(), buffer);
    }
}

// Returns true if the pipeline should render the given entity, false otherwise
fn filter<M: Material>(
    mesh: &Mesh,
    renderer: &Renderer,
) -> bool {
    let enabled = renderer.visible;
    let attribute = mesh.vertices().enabled().contains(M::attributes());
    let validity = mesh.vertices().len().is_some();
    enabled && validity && attribute
}

// Render all the visible surfaces of a specific material type
pub(super) fn render_surfaces<'r, M: Material>(
    world: &'r World,
    meshes: &'r Storage<Mesh>,
    pipeline: &'r GraphicsPipeline<SwapchainFormat, ()>,
    default: &'r DefaultMaterialResources,
    render_pass: &mut ActiveRenderPass<'r, '_, '_, SwapchainFormat, ()>
) {
    // Get a rasterizer for the current render pass by binding a pipeline
    let (mut active, mut bindings) = render_pass.bind_pipeline(pipeline);
    let supported = M::attributes();

    // Get the material storage and resources for this material
    let materials = world.get::<Storage<M>>().unwrap();
    let mut resources = M::fetch(world);

    // Set the global material bindings    
    M::set_global_bindings(&mut resources, default, &mut bindings);
 

    // Get all the entities that contain a visible surface
    let scene = world.get::<Scene>().unwrap();
    let query = scene.query::<(&Surface<M>, &Renderer)>();


    // Iterate over all the surface of this material
    for (surface, renderer) in query {
        // Get the mesh and material that correspond to this surface
        let mesh = meshes.get(&surface.mesh);
        let material = materials.get(&surface.material);

        // Skip rendering if not needed
        if !filter::<M>(mesh, renderer) {
            continue;
        }

        // Bind the mesh's vertex buffers
        use crate::attributes::*;
        set_vertex_buffer_attribute::<Position>(supported, mesh, &mut active);
        set_vertex_buffer_attribute::<Normal>(supported, mesh, &mut active);
        set_vertex_buffer_attribute::<Tangent>(supported, mesh, &mut active);
        set_vertex_buffer_attribute::<TexCoord>(supported, mesh, &mut active);

        // Set the index buffer
        let triangles = mesh.triangles();
        active.set_index_buffer(triangles.buffer());

        // Draw the triangulated mesh 
        let indices = 0..(triangles.buffer().len() as u32);
        active.draw_indexed(indices, 0..1);
    }
}
