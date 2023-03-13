use crate::{
    attributes::{Position, RawPosition},
    ActiveScenePipeline, ActiveSceneRenderPass,
    DefaultMaterialResources, EnabledMeshAttributes, Material, Mesh,
    MeshAttribute, Renderer, SceneColor, SceneDepth, Surface, ActiveShadowGraphicsPipeline,
};
use ecs::Scene;
use graphics::{
    ActiveGraphicsPipeline, ActiveRenderPass, Depth, Graphics,
    GraphicsPipeline, PushConstants, SwapchainFormat, Vertex, XYZ, GpuPod, ModuleVisibility,
};
use utils::{Handle, Storage, Time};
use world::World;

// Returns true if the entity should cast shadows, false otherwise
fn filter(mesh: &Mesh, renderer: &Renderer) -> bool {
    let enabled = renderer.visible;
    let attribute =
        mesh.vertices().enabled().contains(EnabledMeshAttributes::POSITIONS);
    let validity = mesh.vertices().len().is_some();
    enabled && validity && attribute
}

// Render all the visible surfaces of a specific material type
pub(super) fn render_shadows<'r, M: Material>(
    world: &'r World,
    meshes: &'r Storage<Mesh>,
    default: &'r DefaultMaterialResources,
    active: &mut ActiveShadowGraphicsPipeline<'_, 'r, '_>,
) {
    // Don't do shit if we won't cast shadows
    if !M::casts_shadows() {
        return;
    }

    // Get all the entities that contain a visible surface
    let scene = world.get::<Scene>().unwrap();
    let query = scene.query::<(&Surface<M>, &Renderer)>();

    // Iterate over all the surfaces of this material
    for (surface, renderer) in query {
        // Get the mesh and material that correspond to this surface
        let mesh = meshes.get(&surface.mesh);

        // Set the push constant ranges right before rendering (in the hot loop!)
        // Skip rendering if not needed
        if !filter(mesh, renderer) {
            continue;
        }

        // Set the mesh matrix push constant
        active.set_push_constants(|constants| {
            let matrix = renderer.matrix;
            let cols = matrix.cols;
            let bytes = GpuPod::into_bytes(&cols);
            constants.push(bytes, 0, ModuleVisibility::Vertex);
        });

        // Set the position buffer
        let positions = mesh.vertices().attribute::<Position>().unwrap();
        active.set_vertex_buffer::<<Position as crate::MeshAttribute>::V>(0, positions, ..).unwrap();

        // Set the index buffer
        let triangles = mesh.triangles();
        active.set_index_buffer(triangles.buffer(), ..).unwrap();

        // Draw the triangulated mesh
        let indices = 0..(triangles.buffer().len() as u32 * 3);
        active.draw_indexed(indices, 0..1);
    }
}
