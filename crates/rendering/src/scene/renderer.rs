use ecs::Entity;
use world::resources::{Handle, Resource};

use crate::{
    material::{AlbedoMap, MaskMap, NormalMap},
    prelude::{Ranged, Texture2D, RGBA},
};

// The global scene renderer that specifies how we should render the surfaces
// This resource will contain the handles to the default PBR textures
// This resource will contain the entity ID of the main camera and the main directional light
#[derive(Resource, Clone)]
#[Locked]
pub struct SceneSettings {
    // Main camera entity that we will use for rendering
    camera: Option<Entity>,

    // Main directional light (sun)
    light: Option<Entity>,

    // Default black and white textures
    /*
    black: Handle<Texture2D<RGBA<Ranged<u8>>>,
    white: Handle<Texture2D<RGBA<Ranged<u8>>>,
    */
    // Default albedo, normal, and mask maps for PBR rendering
    albedo_map: Handle<AlbedoMap>,
    normal_map: Handle<NormalMap>,
    mask_map: Handle<MaskMap>,
}

impl SceneSettings {
    // Are we allowed to render the scene (check if the SceneRenderer is valid)
    pub fn can_render(&self) -> bool {
        self.camera.is_some() && self.light.is_some()
    }

    // Get the main camera entity ID
    pub fn main_camera(&self) -> Option<Entity> {
        self.camera
    }

    // Set the main camera entity ID
    pub fn set_main_camera(&mut self, entity: Entity) {
        self.camera = Some(entity);
    }

    // Get the main light entity ID
    pub fn main_directional_light(&self) -> Option<Entity> {
        self.light
    }

    // Set the main directional light entity ID
    pub fn set_main_directional_light(&mut self, entity: Entity) {
        self.light = Some(entity);
    }

    // Get the handle for the default albedo map
    pub fn albedo_map(&self) -> Handle<AlbedoMap> {
        self.albedo_map.clone()
    }

    // Get the handle for the default normal map
    pub fn normal_map(&self) -> Handle<NormalMap> {
        self.normal_map.clone()
    }

    // Get the handle for the default mask map
    pub fn mask_map(&self) -> Handle<MaskMap> {
        self.mask_map.clone()
    }
}
