use crate::{
    attributes::{Position, RawPosition},
    ActiveScenePipeline, ActiveSceneRenderPass,
    DefaultMaterialResources, EnabledMeshAttributes, Material, Mesh,
    MeshAttribute, Renderer, SceneColor, SceneDepth, Surface, ActiveShadowGraphicsPipeline,
};
use ecs::Scene;
use graphics::{
    ActiveGraphicsPipeline, ActiveRenderPass, Depth, Graphics,
    GraphicsPipeline, PushConstants, SwapchainFormat, Vertex, XYZ,
};
use utils::{Handle, Storage, Time};
use world::World;

// Returns true if the pipeline should render the given entity, false otherwise
fn filter<M: Material>(mesh: &Mesh, renderer: &Renderer) -> bool {
    let enabled = renderer.visible;
    let attribute =
        mesh.vertices().enabled().contains(M::attributes());
    let validity = mesh.vertices().len().is_some();
    enabled && validity && attribute
}

// Render all the visible surfaces of a specific material type
pub(super) fn render_shadows<'r, M: Material>(
    world: &'r World,
    meshes: &'r Storage<Mesh>,
    default: &'r DefaultMaterialResources,
    pipeline: &'r GraphicsPipeline<SceneColor, SceneDepth>,
    active: &mut ActiveShadowGraphicsPipeline<'_, 'r, '_>,
) {
    // Get the material storage and resources for this material
    let meshes = world.get::<Storage<Mesh>>().unwrap();

    // Get all the entities that contain a visible surface
    let scene = world.get::<Scene>().unwrap();
    let query = scene.query::<(&Surface<M>, &Renderer)>();

    // Iterate over all the surface of this material
    /*
    for (surface, renderer) in query {
        // Get the mesh and material that correspond to this surface
        let mesh = meshes.get(&surface.mesh);

        // Set the push constant ranges right before rendering (in the hot loop!)
        /*
        active.set_push_constants(|push_constants| {

        });
        */

        // Set the index buffer
        let triangles = mesh.triangles();
        active.set_index_buffer(triangles.buffer(), ..);

        // Draw the triangulated mesh
        let indices = 0..(triangles.buffer().len() as u32 * 3);
        active.draw_indexed(indices, 0..1);
    }
    */
}
