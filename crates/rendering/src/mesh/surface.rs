use super::SubMesh;
use crate::{
    context::Graphics,
    material::{Material, PropertyBlock},
    shader::Shader,
};
use ecs::{Component, EcsManager};
use math::Transform;
use world::resources::{Handle, Storage};

// A surface is a combination of a sub mesh and a specific material handle
// A renderable entity will have multiple surface sets
#[derive(Component)]
pub struct Surface<M: Material>(Handle<SubMesh>, Handle<M>);

impl<M: Material> Surface<M> {
    // Create a new surface using a material handle and a submesh handle
    pub fn new(submesh: Handle<SubMesh>, material: Handle<M>) -> Self {
        Self(submesh, material)
    }

    // Get the submesh handle
    pub fn submesh(&self) -> &Handle<SubMesh> {
        &self.0
    }

    // Get the material handle
    pub fn material(&self) -> &Handle<M> {
        &self.1
    }
}

// This will render all the surfaces of a unique material type that exist currently
// This function will be internally called by the MaterialRenderer::draw() method each frame
pub(crate) fn render<'a, M: Material + PropertyBlock<'a>>(
    ecs: &'a EcsManager,
    graphics: &'a mut Graphics,
    materials: &'a Storage<M>,
    shaders: &'a Storage<Shader>,
    resources: M::Resources,
) {
    // Fetch all the entities that contain a surface with this material
    let query = ecs.try_view::<(&Transform, &Surface<M>)>().unwrap();

    // Get the device canvas and the scene rasterizer
    let Graphics(device, context) = graphics;
    let canvas = device.canvas_mut();
    let rasterizer = canvas.rasterizer(todo!(), context);

    /*
    for (transform, surface) in query {
        // Fetch the material instance for this surface
        let material = storage.get(surface.material());

        // Calculate world mesh matrix
        let matrix = transform.matrix();

        // Render

    }
    */
}
