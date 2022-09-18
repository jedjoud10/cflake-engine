use crate::{
    material::{Material}, scene::{ShadowMapping, ClusteredShading, DirectionalLight, Renderer, RenderedFrameStats}, mesh::{Mesh, Surface}, shader::Shader, display::{Viewport, Display, RasterSettings, PrimitiveMode}, prelude::Texture, others::Comparison, context::Context,
};

use ecs::Scene;
use math::Rotation;
use world::{World, Storage};

// Render to the main global shadow map if the material is a shadow caster
pub(crate) fn render_shadows<M: for<'w> Material<'w>>(world: &mut World) {
    if !M::shadow_caster() {
        return;
    }

    // Fetch the main resources from the world
    let ecs = world.get::<Scene>().unwrap();
    let meshes = world.get::<Storage<Mesh>>().unwrap();
    let mut _shadow_mapper = world.get_mut::<ShadowMapping>().unwrap();
    let shadow_mapper = &mut *_shadow_mapper;
    let mut context = world.get_mut::<Context>().unwrap();
    let mut stats = world.get_mut::<RenderedFrameStats>().unwrap();
    stats.unique_materials_shadow_casters += 1;
    let ctx = &mut *context;

    // Get the painter and the depth texture used
    let depth = &mut shadow_mapper.depth_tex;
    let painter = &mut shadow_mapper.painter;
    let viewport = Viewport {
        origin: vek::Vec2::zero(),
        extent: depth.dimensions(),
    };
    
    // Rasterizer shadow settings
    let settings = RasterSettings {
        depth_test: Some(Comparison::Less),
        scissor_test: None,
        primitive: PrimitiveMode::Triangles { cull: None },
        srgb: false,
        blend: None,
    };

    // Find all the surfaces that use this material type
    let query = ecs.view::<(&Renderer, &Surface<M>)>().unwrap();
    let query = query.filter(|(renderer, surface)| {
        // Check if the renderer is even enabled
        let enabled = renderer.enabled;

        // Check if the mesh meets the material requirements
        let mesh = meshes.get(&surface.mesh());
        let buffers = mesh.vertices().layout().contains(M::requirements())
            && mesh.vertices().len().is_some();

        enabled && buffers
    });

    // Create a scoped painter and it's rasterizer
    let mut scoped = painter.scope(viewport, (), depth.mip_mut(0).unwrap(), ()).unwrap();
    let (mut rasterizer, mut uniforms) = scoped.rasterizer(ctx, &mut shadow_mapper.shader, settings);

    // Set the global uniforms
    uniforms.set_mat4x4("lightspace_matrix", shadow_mapper.proj_matrix * shadow_mapper.view_matrix);

    // Render each mesh as if it was a shadow caster
    for (renderer, surface) in query {
        let mesh = meshes.get(&surface.mesh());
        uniforms.set_mat4x4("world_matrix", renderer.matrix);
        rasterizer.draw(mesh, unsafe {
            uniforms.assume_valid()
        });
        stats.shadow_casters_surfaces += 1;
        stats.shadow_casters_verts += mesh.vertices().len().unwrap() as u32;
        stats.shadow_casters_tris += mesh.triangles().len() as u32;
    }
}