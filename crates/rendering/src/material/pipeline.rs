use super::{AlbedoMap, Material};
use crate::{
    canvas::{PrimitiveMode, RasterSettings, Canvas},
    context::{Context, Window},
    mesh::{Mesh, Surface},
    prelude::{Shader, Uniforms},
    scene::{Camera, Directional, Renderer, SceneSettings}, buffer::ElementBuffer,
};
use assets::Assets;
use ecs::EcsManager;
use math::{Location, Rotation};
use std::{any::type_name, marker::PhantomData};
use world::{Handle, Read, Resource, Storage, World};

// Statistics that tell us what exactly happened when we rendered the material surfaces through the pipeline
#[derive(Debug)]
pub struct PipelineStats {
    triangles: u128,
    vertices: u128,
    material_instance_calls: u128,
    mesh_draw_calls: u128,
    material_name: &'static str,
}

// Marker that tells us that we have a registered valid pipeline
pub struct PipeId<M: for<'w> Material<'w>>(pub(crate) PhantomData<Pipeline<M>>);

impl<M: for<'w> Material<'w>> Clone for PipeId<M> {
    fn clone(&self) -> Self {
        Self(PhantomData::default())
    }
}

impl<M: for<'w> Material<'w>> Copy for PipeId<M> {}

// Pipeline trait that will be boxed and stored from within the world
// TODO: Redesign to allow for user defined pipelines
pub(crate) trait SpecializedPipeline: 'static {
    fn render(&self, world: &mut World) -> PipelineStats;
}

// Main material pipeline that shall use one single material shader
pub struct Pipeline<M: for<'w> Material<'w>> {
    pub(crate) shader: Handle<Shader>,
    pub(crate) _phantom: PhantomData<M>,
}

impl<M: for<'w> Material<'w>> SpecializedPipeline for Pipeline<M> {
    fn render(&self, world: &mut World) -> PipelineStats {
        let scene = world.get::<SceneSettings>().unwrap();
        let ecs = world.get::<EcsManager>().unwrap();
        let materials = world.get::<Storage<M>>().unwrap();
        let meshes = world.get::<Storage<Mesh>>().unwrap();
        let window = world.get::<Window>().unwrap();
        let mut shaders = world.get_mut::<Storage<Shader>>().unwrap();
        let mut ctx = world.get_mut::<Context>().unwrap();
        let mut canvases = world.get_mut::<Storage<Canvas>>().unwrap();
        let mut property_block_resources = M::fetch(world);

        // How exactly we should rasterize the surfaces
        let settings: RasterSettings = RasterSettings {
            depth_test: M::depth_comparison(),
            scissor_test: None,
            primitive: M::primitive_mode(),
            srgb: M::srgb(),
            blend: M::blend_mode(),
        };

        // Fetch the shader and enable stats
        let shader = shaders.get_mut(&self.shader);
        let mut stats = PipelineStats {
            triangles: 0,
            vertices: 0,
            material_instance_calls: 0,
            mesh_draw_calls: 0,
            material_name: type_name::<M>(),
        };

        // Find all the surfaces that use this material type (and that have a valid renderer and valid mesh)
        let query = ecs.view::<(&Renderer, &Surface<M>)>().unwrap();
        let query = query.filter(|(renderer, surface)| {
            let renderer = renderer.enabled();
            let mesh = meshes.get(&surface.mesh());
            let buffers = mesh.vertices().layout().contains(M::requirements()) && mesh.vertices().len().is_some();
            renderer && buffers
        });

        // Get the main camera component (there has to be one for us to render)
        let camera_entry = ecs.entry(scene.main_camera().unwrap()).unwrap();
        let camera = camera_entry.as_view::<(&Camera, &Location, &Rotation)>().unwrap();

        // Get the main directional light
        let light_entry = ecs.entry(scene.main_directional_light().unwrap()).unwrap();
        let light = light_entry.as_view::<(&Directional, &Rotation)>().unwrap();

        // Create a new rasterizer so we can draw the objects onto the world
        let (mut rasterizer, mut uniforms) = canvases.get_mut(&scene.canvas()).rasterizer(&mut ctx, shader, settings);

        M::set_static_properties(
            &mut uniforms,
            &mut property_block_resources,
            rasterizer.canvas(),
            &scene,
            camera,
            light,
        );

        // Render each surface that is present in the query
        let mut old: Option<Handle<M>> = None;
        for (renderer, surface) in query {
            // Check if we changed material instances
            if old != Some(surface.material().clone()) {
                old = Some(surface.material().clone());
                stats.material_instance_calls += 1;
                let instance = materials.get(old.as_ref().unwrap());

                // Update the material property block uniforms
                M::set_instance_properties(
                    instance,
                    &mut uniforms,
                    &mut property_block_resources,
                    &scene,
                    camera,
                    light,
                );
            }

            // Set the uniforms per renderer
            M::set_render_properties(
                &mut uniforms,
                &mut property_block_resources,
                renderer,
                camera,
                light,
            );

            // Draw the surface object using the current rasterizer pass
            let mesh = meshes.get(&surface.mesh());

            rasterizer.draw(mesh, unsafe { uniforms.assume_valid() });
            stats.mesh_draw_calls += 1;
            stats.vertices += mesh.vertices().len().unwrap() as u128;
            stats.triangles += mesh.triangles().len() as u128;
        }
        stats
    }
}
