use assets::Assets;
use ecs::EcsManager;
use math::Transform;
use world::{Handle, Storage, Read};

use crate::{
    canvas::Canvas,
    context::{Context, Window},
    mesh::Mesh,
    scene::{Camera, Directional, Renderer, SceneSettings},
    shader::{FragmentStage, Processor, Shader, ShaderCompiler, Uniforms, VertexStage},
    texture::{Ranged, Texture, Texture2D, RG, RGB, RGBA},
};

use super::Material;

// Albedo map (color data), rgba
pub type AlbedoMap = Texture2D<RGBA<Ranged<u8>>>;

// Normal map (bumps), rgb
pub type NormalMap = Texture2D<RGB<Ranged<u8>>>;

// Mask map (r = roughness, g = metallic), rg
pub type MaskMap = Texture2D<RG<Ranged<u8>>>;

// A standard Physically Based Rendering material that we will use by default
// PBR Materials try to replicate the behavior of real light for better graphical fidelty and quality
pub struct Standard {
    albedo: Option<Handle<AlbedoMap>>,
    normal: Option<Handle<NormalMap>>,
    mask: Option<Handle<MaskMap>>,
    bumpiness: f32,
    roughness: f32,
    metallic: f32,
    tint: vek::Rgb<f32>,
}

impl<'w> Material<'w> for Standard {
    type Resources = (
        Read<'w, Storage<AlbedoMap>>,
        Read<'w, Storage<NormalMap>>,
        Read<'w, Storage<MaskMap>>,
    );

    fn fetch(
            world: &'w world::World,
        ) -> Self::Resources {
        let albedo_map = world.get::<Storage<AlbedoMap>>().unwrap();
        let normal_map = world.get::<Storage<NormalMap>>().unwrap();
        let mask_map = world.get::<Storage<MaskMap>>().unwrap();
        (albedo_map, normal_map, mask_map)
    }

    // This method will be called once right before we start rendering the batches
    fn set_static_properties<'u>(
        uniforms: &mut Uniforms<'u>,
        _resources: &mut Self::Resources,
        _canvas: &Canvas,
        _scene: &SceneSettings,
        camera: (&Camera, &Transform),
        light: (&Directional, &Transform),
    ) {
        uniforms.set_mat4x4("view_matrix", camera.0.view());
        uniforms.set_mat4x4("proj_matrix", camera.0.projection());
        uniforms.set_vec3("camera", camera.1.position);
        uniforms.set_vec3("forward", camera.1.forward());
        uniforms.set_vec3("light_dir", light.1.forward());
    }

    // This method will be called for each surface that we have to render
    fn set_render_properties(
        uniforms: &mut Uniforms,
        _resources: &mut Self::Resources,
        renderer: &Renderer,
        _camera: (&Camera, &Transform),
        _light: (&Directional, &Transform),
    ) {
        uniforms.set_mat4x4("world_matrix", renderer.matrix());
    }

    // This method will be called whenever we detect a material instance change
    fn set_instance_properties(
        &self,
        uniforms: &mut Uniforms,
        resources: &mut Self::Resources,
        scene: &SceneSettings,
        _camera: (&Camera, &Transform),
        _light: (&Directional, &Transform),
    ) {
        let (albedo_maps, normal_maps, mask_maps) = resources;

        fn fallback<'a, T: 'static>(
            storage: &'a Storage<T>,
            opt: &Option<Handle<T>>,
            fallback: Handle<T>,
        ) -> &'a T {
            opt.as_ref()
                .map(|handle| storage.get(handle))
                .unwrap_or_else(|| storage.get(&fallback))
        }

        uniforms.set_vec3("tint", self.tint);
        uniforms.set_scalar("bumpiness", self.bumpiness);
        uniforms.set_scalar("roughness", self.roughness);
        uniforms.set_scalar("metallic", self.metallic);

        let albedo_map = fallback(albedo_maps, &self.albedo, scene.missing());
        let normal_map = fallback(normal_maps, &self.normal, scene.normal_map());
        let mask_map = fallback(mask_maps, &self.mask, scene.mask_map());

        uniforms.set_sampler("albedo", albedo_map);
        uniforms.set_sampler("normal", normal_map);
        uniforms.set_sampler("mask", mask_map);
    }

    fn shader(ctx: &mut Context, assets: &mut Assets) -> Shader {
        let vs = assets
            .load::<VertexStage>("engine/shaders/pbr.vrsh.glsl")
            .unwrap();

        let fs = assets
            .load::<FragmentStage>("engine/shaders/pbr.frsh.glsl")
            .unwrap();

        ShaderCompiler::link((vs, fs), Processor::new(assets), ctx)
    }
}

impl Standard {
    // Create a new standard builder with default parameters
    pub fn builder() -> StandardBuilder {
        StandardBuilder(Self {
            albedo: None,
            normal: None,
            mask: None,
            bumpiness: 1.0,
            roughness: 1.0,
            metallic: 0.0,
            tint: vek::Rgb::white(),
        })
    }
}

// This is a builder that we can use to optionally set some material parameters
pub struct StandardBuilder(Standard);

impl StandardBuilder {
    // Set the albedo map
    pub fn with_albedo(mut self, albedo: &Handle<AlbedoMap>) -> Self {
        self.0.albedo = Some(albedo.clone());
        self
    }

    // Set the normal map
    pub fn with_normal(mut self, normal: &Handle<NormalMap>) -> Self {
        self.0.normal = Some(normal.clone());
        self
    }

    // Set the mask map
    pub fn with_mask(mut self, mask: &Handle<MaskMap>) -> Self {
        self.0.mask = Some(mask.clone());
        self
    }

    // Set the tint parameter
    pub fn with_tint(mut self, tint: vek::Rgb<f32>) -> Self {
        self.0.tint = tint;
        self
    }

    // Set the bumpiness parameter
    pub fn with_bumpiness(mut self, bumpiness: f32) -> Self {
        self.0.bumpiness = bumpiness;
        self
    }

    // Set the roughness parameter
    pub fn with_roughness(mut self, roughness: f32) -> Self {
        self.0.roughness = roughness;
        self
    }

    // Set the metallic parameter
    pub fn with_metallic(mut self, metallic: f32) -> Self {
        self.0.metallic = metallic;
        self
    }

    // Return the inner material stored within the builder
    pub fn build(self) -> Standard {
        self.0
    }
}
