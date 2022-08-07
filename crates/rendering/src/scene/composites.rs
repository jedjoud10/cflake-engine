use ecs::Entity;
use math::Location;
use world::Handle;

use crate::{canvas::{Canvas, ColorAttachment, DepthAttachment}, buffer::UniformBuffer, prelude::{Uniforms, Shader}, mesh::Mesh, context::Context};

use super::PointLight;

// Clustered shading is a method to render multiple lights
// efficienty without losing image quality
// The principle of "Clustered Shading" is to subdivide the camera's view frustum
// into multiple sub-regions called "clusters", and have the lights within them rendered
// TODO: Actually implement this lul
pub struct ClusteredShading {
    pub(crate) main_camera: Option<Entity>,
    pub(crate) main_directional_light: Option<Entity>,
    pub(crate) canvas: Canvas,
    pub(crate) color: Handle<ColorAttachment>,
    pub(crate) depth: Handle<DepthAttachment>,
    pub(crate) point_lights: UniformBuffer<(PointLight, Location)>
}

impl ClusteredShading {
    // Get the main camera entity
    pub fn main_camera(&self) -> Option<Entity> {
        self.main_camera
    }

    // Get the main directional light entity
    pub fn main_directional_light(&self) -> Option<Entity> {
        self.main_directional_light
    }
    
    // Get an immutable reference to the renderer's canvas
    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    // Get a mutable reference to the renderer's canvas
    pub fn canvas_mut(&mut self) -> &mut Canvas {
        &mut self.canvas
    }

    // Get the handle to the main color attachment
    pub fn image(&self) -> Handle<ColorAttachment> {
        self.color.clone()
    }
    
    // Get the handle to the main depth attachment
    pub fn depth(&self) -> Handle<DepthAttachment> {
        self.depth.clone()
    }
}

// This is a collection of post-processing effects that will 
// be rendered onto the screen after we render the basic scene
pub struct PostProcessing {
    pub tonemapping_strength: f32,
    pub exposure: f32,
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
    pub(crate) tris: u32,
    pub(crate) verts: u32,
    pub(crate) unique_materials: u32,
    pub(crate) material_instances: u32,
    pub(crate) surfaces: u32,
    pub(crate) current: bool,
}

impl RenderedFrameStats {
    // Get the total number of triangles that we rendered
    pub fn triangle_count(&self) -> u32 {
        self.tris
    }
    
    // Get the total number of vertices that we rendered
    pub fn vertex_count(&self) -> u32 {
        self.verts
    }

    // Get the total number of materials that we rendered
    pub fn unique_material_count(&self) -> u32 { 
        self.unique_materials
    }

    // Get the number of material instances that we used
    pub fn material_instance_count(&self) -> u32 {
        self.material_instances
    }    
    
    // Get the number of surfaces that we rendered
    pub fn surface_count(&self) -> u32 {
        self.surfaces
    }

    // Did we finish rendering this frame?
    pub fn has_finished_rendering(&self) -> bool {
        self.current
    }
}