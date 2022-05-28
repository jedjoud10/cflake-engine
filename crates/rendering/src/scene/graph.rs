use std::rc::Rc;

use crate::{material::Material, mesh::SubMesh, context::Context, raster::Rasterizer};

// A trait that will be implemented for objects that can be drawed onto the screen, like surface clusters or instanced surface clusters
pub trait Draw<'object> {    
    // This will cull any objects that we must not draw
    fn cull(frustum: f32);

    // This will draw all of the objects using the rasterizer
    fn draw(&self, ctx: &mut Context, rasterizer: &mut Rasterizer);
}

// A simple model that uses a unique material
pub struct Model<'object> {
    submeshes: &'object [SubMesh],
    matrix: &'object vek::Mat4<f32>,
}

// A renderer component
pub struct Renderer<M: Material> {
    submeshes: Vec<SubMesh>,
}

// A complex scene graph that contains all the models, lights, shadow casters and surfaces that we must draw
pub struct Graph<'object> {
    models: Vec<Model<'object>>,
}

impl<'object> Graph<'object> {
    // This will draw a whole scene graph onto a canvas rasterizer
    pub fn draw(&mut self, ctx: &mut Context, rasterizer: &mut Rasterizer) {

    }
}