use crate::{mesh::{Mesh, SubMesh}, material::Material};
/*
// A surface that we will draw onto the screen
pub struct Surface<'object> {
    submesh: &'object SubMesh,
    material: &'object dyn Material,
}
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