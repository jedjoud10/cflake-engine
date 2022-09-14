use ecs::Entity;
use math::Location;

use crate::{
    buffer::UniformBuffer,
    mesh::Mesh,
    painter::Painter,
    prelude::{Depth, Ranged, Shader, Texture2D, RGB, SRGB},
};

use super::PointLight;

// Clustered shading is a method to render multiple lights
// efficienty without losing image quality
// The principle of "Clustered Shading" is to subdivide the camera's view frustum
// into multiple sub-regions called "clusters", and have the lights within them rendered
// TODO: Actually implement this lul
pub struct ClusteredShading {
    pub(crate) main_camera: Option<Entity>,
    pub(crate) skysphere_entity: Option<Entity>,
    pub(crate) painter: Painter<RGB<f32>, Depth<Ranged<u32>>, ()>,
    pub(crate) color_tex: Texture2D<RGB<f32>>,
    pub(crate) depth_tex: Texture2D<Depth<Ranged<u32>>>,
    pub(crate) main_directional_light: Option<Entity>,
    pub(crate) point_lights: UniformBuffer<(PointLight, Location)>,
}

impl ClusteredShading {
    // Get the main camera entity
    pub fn main_camera(&self) -> Option<Entity> {
        self.main_camera
    }

    // Get the main sky entity
    pub fn sky_entity(&self) -> Option<Entity> {
        self.skysphere_entity
    }

    // Get the main directional light entity
    pub fn main_directional_light(&self) -> Option<Entity> {
        self.main_directional_light
    }
}

// This is a collection of post-processing effects that will
// be rendered onto the screen after we render the basic scene
pub struct PostProcessing {
    pub tonemapping_strength: f32,
    pub exposure: f32,
    pub gamma: f32,
    pub vignette_strength: f32,
    pub vignette_size: f32,
    pub bloom_radius: f32,
    pub bloom_strength: f32,
    pub bloom_contrast: f32,
}

// The compositor is what we shall use to combine the clustered shading canvas and other composites
pub(crate) struct Compositor {
    pub(crate) quad: Mesh,
    pub(crate) compositor: Shader,
}

// These settings keep track what we rendered within a single frame
#[derive(Default, Debug, Clone, Copy)]
pub struct RenderedFrameStats {
    pub tris: u32,
    pub verts: u32,
    pub unique_materials: u32,
    pub material_instances: u32,
    pub rendered_surfaces: u32,
    pub invisible_surfaces: u32,
    pub current: bool,
}
