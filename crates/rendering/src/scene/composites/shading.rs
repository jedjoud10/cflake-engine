use ecs::Entity;

use crate::{material::{AlbedoMap, NormalMap}, canvas::Canvas};

// Clustered shading is a method to render multiple lights
// efficienty without losing image quality
// The principle of "Clustered Shading" is to subdivide the camera's view frustum
// into multiple sub-regions called "clusters", and have the lights within them rendered
// TODO: Actually implement this lul
pub struct ClusteredShading {
    pub(crate) main_camera: Option<Entity>,
    pub(crate) main_directional_light: Option<Entity>,
    pub(crate) canvas: Canvas,
}