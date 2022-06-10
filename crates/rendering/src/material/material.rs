use std::{any::TypeId, marker::PhantomData, rc::Rc};

use ahash::AHashMap;
use assets::loader::AssetLoader;
use ecs::EcsManager;
use world::{resources::{Handle, ResourceSet, Storage}, World};

use crate::{
    context::{Context, Device, Graphics},
    mesh::SubMesh,
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
    pub fn build(self, ctx: &mut Context) -> M {
        // Add the material type renderer to the context
        //ctx.register_batch_renderer(todo!());

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
    type Render: MaterialRenderer; 

    // Create a default material instance
    fn default(id: InstanceID<Self>) -> Self; 

    // Create a new material renderer for this material type
    fn renderer() -> Self::Render;

    // Create a new instance builder for this material type
    fn builder() -> InstanceBuilder<Self> {
        InstanceBuilder::default()
    }

    // Get the current material instance ID
    fn instance(&self) -> &InstanceID<Self>;

    // Load a new instance of the shader that we will use for this material
    fn load_shader(ctx: &mut Context, loader: &mut AssetLoader) -> Shader;
}

// Uniforms setter. I have to separate Material and UniformsSetter because it will cause weird lifetime fuckery if I don't
pub trait PropertyBlock<'a>: Sized {
    // The resources that we might need from the world to set the uniforms
    type Resources: 'a;

    // Load the valid resources from the resource set, whilst also fetching the necessary values for rendering
    fn fetch_resources(
        world: &'a mut World,
    ) -> (
        &'a EcsManager,
        &'a mut Graphics,
        &'a Storage<Self>,
        &'a Storage<Shader>,
        Self::Resources,
    );

    // Set the corresponding uniforms for this block using the resources that we loaded in
    fn set_uniforms(&self, uniforms: &mut Uniforms, res: Self::Resources);
}

// A material renderer will simply take the world and try to render all the surface that make up the render objects
// This trait will be automatically implemented for BatchRenderer (since we can batch all the surface into one shader use pass)
pub trait MaterialRenderer: 'static {
    // Render all the objects that use this material type
    // Get all the renderers that use this material type
    //   Fetch their material instances, per object
    //   Set the required uniforms (transform, matrices)
    //   Set the material uniforms (M::set)
    //   Render the object
    fn render(&self, world: &mut World);
}

// A batch renderer will use a single shader use pass to render the materialized surfaces
pub struct BatchRenderer<M: Material> {
    material: PhantomData<M>,
    shader: Handle<Shader>,
    layer: i32,
} 


impl<'a, M: Material + PropertyBlock<'a>> MaterialRenderer for BatchRenderer<M> {
    fn render(&self, world: &mut World) {
        let (ecs, graphics, materials, shader, prop_block_resources) = M::fetch_resources(world);
    }
}