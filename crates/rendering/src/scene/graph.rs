use crate::{material::Material, mesh::SubMesh, context::Context, framebuffer::Canvas};

// A trait that will be implemented for objects that can be drawed onto the screen, like surface clusters or instanced surface clusters
pub trait Draw<'object> {    
    // This will cull any objects that we must not draw
    fn cull(frustum: f32);

    // This will draw all of the objects
    fn draw(&self, ctx: &mut Context, canvas: &mut Canvas);
}

// A model cluster contains multiples surfaces and their 

/*
/*
// A simple model that we will render
pub struct Mode<'object> {
    surfaces: &'scene [Surface<'object>]
    matrix: &'object vek::Mat4<f32>,
}
*/
// A scene graph that contains all the objects and lights that we must draw onto the screen
pub struct SceneGraph<'scene> {
    visible: Vec<Surface<'scene>>,
}
*/