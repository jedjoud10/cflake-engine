use std::marker::PhantomData;

use assets::Assets;
use ecs::EcsManager;
use math::Transform;
use world::{Handle, Storage, World};

use crate::{
    canvas::{FaceCullMode, PrimitiveMode, RasterSettings, Canvas},
    context::{Context, Graphics, Device},
    mesh::{SubMesh, Surface},
    others::Comparison,
    scene::{Camera, Renderer, SceneSettings, Directional},
    shader::{Shader, Uniforms},
};

use super::{InstanceID, MaterialBuilder};

// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
pub trait Material: 'static + Sized {
    // Create a default material instance
    fn default(id: InstanceID<Self>) -> Self;

    /*
    // Create a new material pipeline for this material type. This will only be called
    fn pipeline(
        ctx: &mut Context,
        loader: &mut Assets,
        storage: &mut Storage<Shader>,
    ) -> Self::Pipe;
    */

    // Create a new instance builder for this material type
    fn builder() -> MaterialBuilder<Self> {
        MaterialBuilder::default()
    }

    // Get the current material instance ID
    fn instance(&self) -> &InstanceID<Self>;
}



// A material pipeline contain the logic telling us how we should render and draw a specific material type
pub trait Pipeline: 'static {
    // Get the required shader for this material pipeline
    fn shader(&self) -> Handle<Shader>;

    // Pepare the pipeline for rendering
    fn prepare(&self) {}

    // Cull any surfaces if needed
    fn cull(&self) {}

    // Render the materialized surface onto the screen
    fn render(&self, world: &mut World) -> Option<Stats>;
}

// A batch pipeline will use a single shader use pass to render the materialized surfaces
pub struct BatchPipeline<M: Material + Batched> {
    shader: Handle<Shader>,
    _phantom: PhantomData<M>,
}


impl<'world, M: Batched + Material + PropertyBlock<'world>> Pipeline for BatchPipeline<M> {
    fn shader(&self) -> Handle<Shader> {
        self.shader.clone()
    }

    fn render(&self, world: &mut World) -> Option<Stats> {
        self.test(world);
        None
    }
}

// This is a batched material that will be used from within batch rendering pipelines
// By default, the Standard PBR material is using batching, however, the user can enable batching for their own materials
pub trait Batched: Material {}

// A property block is an interface that tells us exactly we should set the material properties
pub trait PropertyBlock<'world>: Sized + Material + Batched {
    // The resources that we need to fetch from the world to set the uniforms
    type PropertyBlockResources: 'world;

    // Fetch the default rendering resources and the material property block resources as well
    fn fetch(
        world: &'world mut World,
    ) -> (
        &'world SceneSettings,
        &'world EcsManager,
        &'world Storage<Self>,
        &'world Storage<SubMesh>,
        &'world mut Storage<Shader>,
        &'world mut Graphics,
        Self::PropertyBlockResources,
    );

    // Set the global and static instance properties when we start batch rendering
    fn set_static_properties<'u>(
        uniforms: &mut Uniforms<'u>,
        resources: &Self::PropertyBlockResources,
        canvas: &Canvas,
        scene: &SceneSettings,
        camera: (&Camera, &Transform),
    ) where 
        'world: 'u {}

    // Set the uniforms for this property block right before we render our surface
    fn set_render_properties<'u>(
        uniforms: &mut Uniforms<'u>,
        resources: &Self::PropertyBlockResources,
        renderer: &Renderer,
        camera: (&Camera, &Transform),
    ) where 
        'world: 'u {}

    // With the help of the fetched resources, set the uniform properties for a unique material instance
    // This will only be called whenever we switch instances
    fn set_instance_properties<'u>(
        &'world self,
        uniforms: &mut Uniforms<'u>,
        resources: &Self::PropertyBlockResources,
        scene: &SceneSettings,
        camera: (&Camera, &Transform),
    ) where
        'world: 'u;
}

// Statistics that tell us what exactly happened when we rendered the material surfaces
pub struct Stats {}

/*
// Fetch the rendering resources to batch render the surfaces
        

*/

/*
        let (scene, ecs, materials, submeshes, shaders, graphics, property_block_resources) =
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
        let shader = shaders.get_mut(&self.shader());

        // Find all the surfaces that use this material type (and that have a valid renderer component)
        let query = ecs.try_view::<(&Renderer, &Surface<M>)>().unwrap();
        let query = query.filter(|(renderer, _)| renderer.enabled());

        // Get the main camera component (there has to be one for us to render)
        let camera_entry = ecs.try_entry(scene.main_camera().unwrap()).unwrap();
        let camera_transform = camera_entry.get::<Transform>().unwrap();
        let camera_data = camera_entry.get::<Camera>().unwrap();
        let camera = (camera_data, camera_transform);

        // Get the main directional light
        let light_entry = ecs.try_entry(scene.main_directional_light().unwrap()).unwrap();
        let light_transform = light_entry.get::<Transform>().unwrap();
        let light = light_entry.get::<Directional>().unwrap();
        dbg!(light_transform.forward());

        // Create a new rasterizer so we can draw the objects onto the world
        let (mut rasterizer, mut uniforms) = device.canvas_mut().rasterizer(ctx, shader, settings);
        M::set_static_properties(&mut uniforms, &property_block_resources, rasterizer.canvas(), scene, camera);        

        // Render each surface that is present in the query
        let mut old: Option<Handle<M>> = None;
        for (renderer, surface) in query {
            // Check if we changed material instances
            if old != Some(surface.material().clone()) {
                old = Some(surface.material().clone());
                let instance = materials.get(old.as_ref().unwrap());
                let _ = instance.instance();
                
                // Update the material property block uniforms
                M::set_instance_properties(instance, &mut uniforms, &property_block_resources, &scene, camera);
            }
            
            // Set the uniforms per renderer
            M::set_render_properties(&mut uniforms, &property_block_resources, renderer, camera);

            // Draw the surface object using the current rasterizer pass
            let submesh = submeshes.get(surface.submesh());
            rasterizer.draw(submesh, &mut uniforms).unwrap();
        }
        None
        */