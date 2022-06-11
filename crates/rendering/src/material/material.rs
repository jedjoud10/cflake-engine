use std::{any::TypeId, marker::PhantomData, rc::Rc};

use ahash::AHashMap;
use assets::Assets;
use ecs::EcsManager;
use math::Transform;
use world::{
    resources::{Handle, ResourceSet, Storage},
    World,
};

use crate::{
    canvas::rasterizer::{FaceCullMode, PrimitiveMode, RasterSettings},
    commons::Comparison,
    context::{Context, Device, Graphics},
    mesh::{SubMesh, Surface},
    scene::{Renderer, Camera},
    shader::{Shader, Uniforms},
};

// Instance builder that will take a unique material and construct a new instance for it
// You can implement the instance builder for your specic material to write some methods that use the builder pattern
pub struct InstanceBuilder<M: Material>(M);

impl<M: Material> Default for InstanceBuilder<M> {
    // Create a default instance builder by creating a new material from it's instance
    fn default() -> Self {
        Self(M::default(InstanceID(Default::default())))
    }
}

impl<M: Material> InstanceBuilder<M> {
    // Get an immutable reference to the underlying material
    pub fn material(&self) -> &M {
        &self.0
    }

    // Get a mutable reference to the underlying material
    pub fn material_mut(&mut self) -> &mut M {
        &mut self.0
    }

    // Build the underlying material instance
    pub fn build(self, ctx: &mut Context, loader: &mut Assets, storage: &mut Storage<Shader>) -> M {
        // Add the material type renderer to the context
        ctx.register_material_renderer::<M, _>(|ctx| M::renderer(ctx, loader, storage));

        // Simply return the built material
        self.0
    }
}

// This is an Instance ID that will be stored within the materials
// By itself it does nothing, but we have to store it since the only way we can generate an instance ID is by using the descriptor
pub struct InstanceID<M: Material>(PhantomData<M>);

// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
pub trait Material: 'static + Sized {
    // How exactly we should render this material
    type Renderer: MaterialRenderer;

    // Create a default material instance
    fn default(id: InstanceID<Self>) -> Self;

    // Create a new material renderer for this material type (PS: this will only be called once)
    fn renderer(
        ctx: &mut Context,
        loader: &mut Assets,
        storage: &mut Storage<Shader>,
    ) -> Self::Renderer;

    // Create a new instance builder for this material type
    fn builder() -> InstanceBuilder<Self> {
        InstanceBuilder::default()
    }

    // Get the current material instance ID
    fn instance(&self) -> &InstanceID<Self>;
}

// A property block is an interface that tells us exactly we should set the material properties
pub trait PropertyBlock<'world>: Sized {
    // The resources that we need to fetch from the world to set the uniforms
    type PropertyBlockResources: 'world;

    // Fetch the default rendering resources and the material property block resources as well
    fn fetch(
        world: &'world mut World,
    ) -> (
        &'world EcsManager,
        &'world Storage<Self>,
        &'world Storage<SubMesh>,
        &'world mut Storage<Shader>,
        &'world mut Graphics,
        Self::PropertyBlockResources,
    );

    // With the help of the fetched resources, set the uniform properties for a unique material instance
    fn set_instance_properties(
        &'world self,
        uniforms: &mut Uniforms,
        resources: &Self::PropertyBlockResources,
    );
}

// Statistics that tell us what exactly happened when we rendered the material surfaces
pub struct Stats {}

// A material renderer will simply take the world and try to render all the surface that make up the render objects
// This trait will be automatically implemented for BatchRenderer (since we can batch all the surface into one shader use pass)
pub trait MaterialRenderer: 'static {
    // Render all the objects that use this material type
    // The rendering is implementation specific, so if the user has some sort of optimizations like culling, it would be executed here
    fn render(&self, world: &mut World) -> Option<Stats>;
}

// A batch renderer will use a single shader use pass to render the materialized surfaces
pub struct BatchRenderer<M: Material> {
    shader: Handle<Shader>,
    material: PhantomData<M>,
}

impl<M: Material> From<Handle<Shader>> for BatchRenderer<M> {
    fn from(shader: Handle<Shader>) -> Self {
        Self {
            material: Default::default(),
            shader,
        }
    }
}

impl<M: Material> BatchRenderer<M> {
    // Get a reference to the shader handle
    pub fn shader(&self) -> &Handle<Shader> {
        &self.shader
    }

    // This method will batch render a ton of surfaces using one material instance only
    // This method can be called within the implementation of render()
    pub fn render_batched_surfaces<'a>(&self, world: &'a mut World) -> Option<Stats>
    where
        M: PropertyBlock<'a>,
    {
        // Fetch the rendering resources to batch render the surfaces
        let (ecs, materials, submeshes, shaders, graphics, property_block_resources) =
            M::fetch(world);

        // Get a valid rasterizer from the graphics
        let Graphics(device, ctx) = graphics;
        let shader = shaders.get_mut(self.shader());
        let mut rasterizer = device.canvas_mut().rasterizer(shader, ctx);

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

        // Find all the surfaces that use this material type (and that have a valid renderer component)
        let query = ecs.try_view::<(&Renderer, &Surface<M>)>().unwrap();

        // Get the main camera component (if there is none, just don't render)
        let (_, camera) = ecs.try_view::<(&Transform, &Camera)>().unwrap().next()?;

        // Ignore invisible surfaces
        let query = query.filter(|(renderer, _)| renderer.is_visible());

        // Render the valid surfaces
        let mut old: Option<Handle<M>> = None;
        for (renderer, surface) in query {
            // Get the shader uniforms since we have to set them for each surface
            let mut uniforms = rasterizer.shader_mut().as_mut().uniforms();

            // Set the default surface uniforms
            uniforms.set_mat4x4("_world_matrix", renderer.matrix());
            uniforms.set_mat4x4("_view_matrix", camera.view());
            uniforms.set_mat4x4("_proj_matrix", camera.projection());

            // Check if we changed material instances
            if old != Some(surface.material().clone()) {
                // We changed instances, so we must re-set the uniform property
                old = Some(surface.material().clone());
                let instance = materials.get(old.as_ref().unwrap());
                let _ = instance.instance();

                // Set the material property block uniforms (only if the instance changes)
                M::set_instance_properties(instance, &mut uniforms, &property_block_resources)
            }

            // Render the surface object
            let submesh = submeshes.get(surface.submesh());
            rasterizer.draw(submesh, &settings);
        }

        None
    }
}
