use super::{Material, PropertyBlock, Standard};
use crate::{
    canvas::{FaceCullMode, PrimitiveMode, RasterSettings},
    context::Graphics,
    mesh::Surface,
    others::Comparison,
    prelude::Shader,
    scene::{Camera, Directional, Renderer},
};
use math::Transform;
use std::{marker::PhantomData, rc::Rc};
use world::{Handle, World};

// Statistics that tell us what exactly happened when we rendered the material surfaces through the pipeline
pub struct Stats {}

// A material renderer is responsible for rendering and drawing surfaces of a specific material onto the screen
// For now, material renderers are implemented as functions that can be called back
pub trait Pipeline: 'static {
    // Create a new pipeline from a shader
    fn new(shader: Handle<Shader>) -> Self
    where
        Self: Sized;

    // Fetch the shader handle from the pipeline
    fn shader(&self) -> Handle<Shader>;

    // Cull all the surfaces that we will render
    fn cull(&self, world: &mut World) {}

    // Render all the materialized surfaces
    fn render(&self, world: &mut World) -> Option<Stats>;

    // Post-render method
    fn cleanup(&self, world: &mut World) {}
}

// The default pipeline that uses one shader pass to render everything
// TODO: Find better name
pub struct BatchedPipeline<M: Material + for<'a> PropertyBlock<'a>> {
    shader: Handle<Shader>,
    _phantom: PhantomData<M>,
}

impl<M: Material + for<'a> PropertyBlock<'a>> Pipeline for BatchedPipeline<M> {
    fn new(shader: Handle<Shader>) -> Self
    where
        Self: Sized,
    {
        Self {
            shader,
            _phantom: Default::default(),
        }
    }

    fn shader(&self) -> Handle<Shader> {
        self.shader.clone()
    }

    fn render(&self, world: &mut World) -> Option<Stats> {
        let (scene, ecs, materials, submeshes, shaders, graphics, mut property_block_resources) =
            <M as PropertyBlock<'_>>::fetch(world);

        // How exactly we should rasterize the surfaces
        let settings: RasterSettings = RasterSettings {
            depth_test: Some(Comparison::Less),
            scissor_test: None,
            primitive: PrimitiveMode::Triangles {
                cull: FaceCullMode::Back(true),
            },
            srgb: false,
            blend: None,
        };

        // Create a valid rasterizer and start rendering
        let Graphics(device, ctx) = graphics;
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
        let (mut rasterizer, mut uniforms) = device.canvas_mut().rasterizer(ctx, shader, settings);
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
                let _ = instance.id();

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
            let submesh = submeshes.get(&surface.submesh());
            rasterizer.draw(submesh, &mut uniforms).unwrap();
        }
        None
    }
}
