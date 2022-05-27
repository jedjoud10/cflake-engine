use crate::{material::Material, mesh::SubMesh};


// A surface is just a submesh that is associated with a material
// Technically, the OpenGL renderer will just need a surface to be able to render anything onto the screen
pub struct Surface<'object, M: Material> {
    submesh: &'object SubMesh,
    material: &'object M,
}
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