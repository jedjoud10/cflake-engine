use ecs::Entity;

use crate::material::{AlbedoMap, NormalMap};

// Clustered shading is a method to render multiple lights
// efficienty without losing image quality
// The principle of "Clustered Shading" is to subdivide the camera's view frustum
// into multiple sub-regions called "clusters", and have the lights within them rendered
// TODO: Actually implement this lul
pub struct ClusteredShading {
    camera: Option<Entity>,
    directional: Option<Entity>,
}

impl ClusteredShading {
    // Set the main camera entity
    pub fn set_main_camera(&mut self, entity: Entity) {
        self.camera = Some(entity)
    }
    
    // Set the directional light entity
    pub fn set_directional_light(&mut self, entity: Entity) {
        self.directional = Some(entity)
    }
    
    // Get the main camera entity
    pub fn main_camera(&self) -> Option<Entity> {
        self.camera
    }
    
    // Get the directional light entity
    pub fn directional_light(&self) -> Option<Entity> {
        self.directional
    }
}