use ecs::Entity;
use world::{Handle, Resource};

use crate::{
    material::{AlbedoMap, MaskMap, NormalMap, Standard},
    mesh::Mesh,
    prelude::{Ranged, Texture2D, RGBA}, canvas::Canvas,
};

type Image = Texture2D<RGBA<Ranged<u8>>>;

// The global scene settings that specifies how we should render the surfaces
// This resource will contain the handles to the default PBR textures
// This resource will contain the entity ID of the main camera and the main directional light
pub struct SceneSettings {
    // Main camera entity that we will use for rendering
    camera: Option<Entity>,

    // Main directional light (sun)
    light: Option<Entity>,

    // Default black and white textures
    black: Handle<Image>,
    white: Handle<Image>,

    // Default albedo, normal, and mask maps for PBR rendering
    albedo_map: Handle<AlbedoMap>,
    normal_map: Handle<NormalMap>,
    mask_map: Handle<MaskMap>,

    // Default missing and debug maps
    missing: Handle<AlbedoMap>,
    debug: Handle<NormalMap>,

    // Default cube and sphere meshes
    cube: Handle<Mesh>,
    sphere: Handle<Mesh>,

    // Main renderer framebuffer
    canvas: Handle<Canvas>, 
}

impl SceneSettings {
    // This creates a new scene settings from just the default texture handles
    pub(super) fn new(
        black: Handle<Image>,
        white: Handle<Image>,
        albedo_map: Handle<AlbedoMap>,
        normal_map: Handle<NormalMap>,
        mask_map: Handle<MaskMap>,
        missing: Handle<AlbedoMap>,
        debug: Handle<NormalMap>,
        cube: Handle<Mesh>,
        sphere: Handle<Mesh>,
        canvas: Handle<Canvas>,
    ) -> Self {
        Self {
            camera: None,
            light: None,
            black,
            white,
            albedo_map,
            normal_map,
            mask_map,
            missing,
            debug,
            cube,
            sphere,
            canvas,
        }
    }

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

    // Get the handle for the default missing map
    pub fn missing(&self) -> Handle<AlbedoMap> {
        self.missing.clone()
    }

    // Get the handle for the debug normal mpa
    pub fn debug(&self) -> Handle<NormalMap> {
        self.debug.clone()
    }

    // Get the handle for the default black texture
    pub fn black(&self) -> Handle<Image> {
        self.black.clone()
    }

    // Get the handle for the default white texture
    pub fn white(&self) -> Handle<Image> {
        self.white.clone()
    }

    // Get the handle for the default cube mesh
    pub fn cube(&self) -> Handle<Mesh> {
        self.cube.clone()
    }

    // Get the handle for the default sphere mesh
    pub fn sphere(&self) -> Handle<Mesh> {
        self.sphere.clone()
    }

    // Get the handle for the 3D rendering canvas
    pub fn canvas(&self) -> Handle<Canvas> {
        self.canvas.clone()
    }
}
