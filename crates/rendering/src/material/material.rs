use std::{any::TypeId, marker::PhantomData, rc::Rc};

use ahash::AHashMap;
use assets::loader::AssetLoader;
use ecs::EcsManager;
use world::resources::{Handle, ResourceSet, Storage};

use crate::{
    context::{Context, Device, Graphics},
    mesh::SubMesh,
    shader::{Shader, Uniforms},
};

// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
pub trait Material: 'static {
    // Load a new instance of the shader that we will use for this material
    fn load_shader(ctx: &mut Context, loader: &mut AssetLoader) -> Shader;
}

// Uniforms setter. I have to separate Material and UniformsSetter because it will cause weird lifetime fuckery if I don't
pub trait PropertyBlock<'a>: Sized {
    // The resources that we might need from the world to set the uniforms
    type Resources: 'a;

    // Load the valid resources from the resource set, whilst also fetching the necessary values for rendering
    fn fetch_resources(
        set: &'a mut ResourceSet,
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

// This struct will act like a bridge between the MaterialRenderer trait and the Context
// These settings will be saved as a Box<dyn MaterialRenderer> within the context, since we need it to render the material surfaces
pub struct RenderDescriptor<M: Material> {
    handle: Handle<Shader>,
    _phantom: PhantomData<Handle<M>>,
}

// A match renderer will simply take all the renderers that use this specific material type and render all of them at the same time
// This trait will be simply implemented for structs that contain the render settings for their material
pub trait MaterialRenderer: 'static {
    // Get the shader that we will use to render the material
    fn shader(&self) -> &Handle<Shader>;

    // Render all the objects that use this material type
    // Get all the renderers that use this material type
    //   Fetch their material instances, per object
    //   Set the required uniforms (transform, matrices)
    //   Set the material uniforms (M::set)
    //   Render the object
    fn render(&self, resources: &mut ResourceSet);
}
