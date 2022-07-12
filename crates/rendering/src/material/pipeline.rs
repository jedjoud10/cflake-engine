use super::Material;
use crate::{
    canvas::{PrimitiveMode, RasterSettings},
    context::Context,
    mesh::{Surface, Mesh},
    prelude::Shader,
    scene::{Camera, Directional, Renderer},
};
use assets::Assets;
use math::Transform;
use std::marker::PhantomData;
use world::{Handle, Resource, Storage, World};

// Statistics that tell us what exactly happened when we rendered the material surfaces through the pipeline
pub struct Stats {}

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
    // Render all the materialized surfaces
    fn render(&self, world: &mut World) -> Option<Stats>;
}

// Main material pipeline that shall use one single material shader
#[derive(Resource)]
pub struct Pipeline<M: for<'w> Material<'w>> {
    pub(crate) shader: Handle<Shader>,
    pub(crate) _phantom: PhantomData<M>,
}

impl<M: for<'w> Material<'w>> SpecializedPipeline for Pipeline<M> {
    fn render(&self, world: &mut World) -> Option<Stats> {
        let (scene, ecs, materials, meshes, shaders, window, ctx, mut property_block_resources) =
            <M as Material<'_>>::fetch(world);

        // How exactly we should rasterize the surfaces
        let settings: RasterSettings = RasterSettings {
            depth_test: M::depth_comparison(),
            scissor_test: None,
            primitive: PrimitiveMode::Triangles {
                cull: M::face_cull_mode(),
            },
            srgb: M::srgb(),
            blend: M::blend_mode(),
        };

        // Create a valid rasterizer and start rendering
        let shader = shaders.get_mut(&self.shader);

        // Find all the surfaces that use this material type (and that have a valid renderer component)
        let query = ecs.try_view::<(&Renderer, &Surface<M>)>().unwrap();
        let query = query.filter(|(renderer, _)| renderer.enabled());

        // Get the main camera component (there has to be one for us to render)
        let camera_entry = ecs.try_entry(scene.main_camera().unwrap()).unwrap();
        let camera_transform = camera_entry.get::<Transform>().unwrap();
        let camera_data = camera_entry.get::<Camera>().unwrap();
        let camera = (camera_data, camera_transform);

        // Get the main directional light
        let light_entry = ecs
            .try_entry(scene.main_directional_light().unwrap())
            .unwrap();
        let light_transform = light_entry.get::<Transform>().unwrap();
        let light_data = light_entry.get::<Directional>().unwrap();
        let light = (light_data, light_transform);

        // Create a new rasterizer so we can draw the objects onto the world
        let (mut rasterizer, mut uniforms) = window.canvas_mut().rasterizer(ctx, shader, settings);
        M::set_static_properties(
            &mut uniforms,
            &mut property_block_resources,
            rasterizer.canvas(),
            scene,
            camera,
            light,
        );

        // Render each surface that is present in the query
        let mut old: Option<Handle<M>> = None;
        for (renderer, surface) in query {
            // Check if we changed material instances
            if old != Some(surface.material().clone()) {
                old = Some(surface.material().clone());
                let instance = materials.get(old.as_ref().unwrap());

                // Update the material property block uniforms
                M::set_instance_properties(
                    instance,
                    &mut uniforms,
                    &mut property_block_resources,
                    scene,
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
            if mesh.buffers.contains(M::required()) {
                unsafe {
                    rasterizer.draw(mesh, &mut uniforms).unwrap();                
                }
            }
        }
        None
    }
}
