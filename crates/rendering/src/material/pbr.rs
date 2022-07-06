use assets::Assets;
use ecs::EcsManager;
use math::Transform;
use world::{Handle, Storage, World};

use crate::{
    canvas::Canvas,
    context::{Context, Device, Graphics},
    mesh::{SubMesh, Surface},
    scene::{Camera, Directional, Renderer, SceneSettings},
    shader::{FragmentStage, Processor, Shader, ShaderCompiler, Uniforms, VertexStage},
    texture::{Ranged, Texture, Texture2D, RG, RGB, RGBA},
};

use super::{BatchedPipeline, Material, Pipeline};

// Albedo map (color data), rgba
pub type AlbedoMap = Texture2D<RGBA<Ranged<u8>>>;

// Normal map (bumps), rgb
pub type NormalMap = Texture2D<RGB<Ranged<u8>>>;

// Mask map (r = roughness, g = metallic), rg
pub type MaskMap = Texture2D<RG<Ranged<u8>>>;

// A standard Physically Based Rendering material that we will use by default
// PBR Materials try to replicate the behavior of real light for better graphical fidelty and quality
pub struct Standard {
    // Texture maps used for rendering
    albedo: Option<Handle<AlbedoMap>>,
    normal: Option<Handle<NormalMap>>,
    mask: Option<Handle<MaskMap>>,

    // Texture parameters
    bumpiness: f32,
    roughness: f32,
    metallic: f32,

    // Unique parameters
    tint: vek::Vec3<f32>,
}

impl<'w> Material<'w> for Standard {
    type Resources = (
        &'w Storage<AlbedoMap>,
        &'w Storage<NormalMap>,
        &'w Storage<MaskMap>,
    );

    type Pipeline = BatchedPipeline<Self>;

    // Create a new batch pipeline for the PBR material
    fn pipeline(
        ctx: &mut Context,
        assets: &mut Assets,
        storage: &mut Storage<Shader>,
    ) -> Self::Pipeline {
        let vs = assets
            .load::<VertexStage>("engine/shaders/pbr.vrsh.glsl")
            .unwrap();

        let fs = assets
            .load::<FragmentStage>("engine/shaders/pbr.frsh.glsl")
            .unwrap();

        let shader = ShaderCompiler::link((vs, fs), Processor::new(assets), ctx);

        let handle = storage.insert(shader);

        BatchedPipeline::new(handle)
    }

    fn fetch(
        world: &'w mut world::World,
    ) -> (
        &'w SceneSettings,
        &'w EcsManager,
        &'w Storage<Self>,
        &'w Storage<SubMesh>,
        &'w mut Storage<Shader>,
        &'w mut Graphics,
        Self::Resources,
    ) {
        let (
            ecs_manager,
            materials,
            submesh,
            shaders,
            graphics,
            albedo_maps,
            normal_maps,
            mask_maps,
            scene,
        ) = world
            .get_mut::<(
                &EcsManager,
                &Storage<Self>,
                &Storage<SubMesh>,
                &mut Storage<Shader>,
                &mut Graphics,
                &Storage<AlbedoMap>,
                &Storage<NormalMap>,
                &Storage<MaskMap>,
                &SceneSettings,
            )>()
            .unwrap();
        (
            scene,
            ecs_manager,
            materials,
            submesh,
            shaders,
            graphics,
            (albedo_maps, normal_maps, mask_maps),
        )
    }

    // This method will be called once right before we start rendering the batches
    fn set_static_properties<'u>(
        uniforms: &mut Uniforms<'u>,
        resources: &mut Self::Resources,
        canvas: &Canvas,
        scene: &SceneSettings,
        camera: (&Camera, &Transform),
        light: (&Directional, &Transform),
    ) where
        'w: 'u,
    {
        uniforms.set_mat4x4("view_matrix", camera.0.view());
        uniforms.set_mat4x4("proj_matrix", camera.0.projection());
        uniforms.set_vec3("camera", camera.1.position);
        uniforms.set_vec3("forward", camera.1.forward());
        uniforms.set_vec3("light_dir", light.1.forward());
    }

    // This method will be called for each surface that we have to render
    fn set_render_properties<'u>(
        uniforms: &mut Uniforms<'u>,
        resources: &mut Self::Resources,
        renderer: &Renderer,
        camera: (&Camera, &Transform),
        light: (&Directional, &Transform),
    ) where
        'w: 'u,
    {
        uniforms.set_mat4x4("world_matrix", renderer.matrix());
    }

    // This method will be called whenever we detect a material instance change
    fn set_instance_properties<'u>(
        &'w self,
        uniforms: &mut Uniforms<'u>,
        resources: &mut Self::Resources,
        scene: &SceneSettings,
        camera: (&Camera, &Transform),
        light: (&Directional, &Transform),
    ) where
        'w: 'u,
    {
        let (albedo_maps, normal_maps, mask_maps) = resources;

        // Fallback to the given handle if the first handle is missing
        fn fallback<'a, T: 'static>(
            storage: &'a Storage<T>,
            opt: &Option<Handle<T>>,
            fallback: Handle<T>,
        ) -> &'a T {
            opt.as_ref()
                .map(|handle| storage.get(handle))
                .unwrap_or_else(|| storage.get(&fallback))
        }

        // Scalar and vec parameters
        uniforms.set_vec3("tint", self.tint);
        uniforms.set_scalar("bumpiness", self.bumpiness);
        uniforms.set_scalar("roughness", self.roughness);
        uniforms.set_scalar("metallic", self.metallic);

        // Try to fetch the textures, and fallback to the default ones if we can't
        let albedo_map = fallback(albedo_maps, &self.albedo, scene.albedo_map());
        let normal_map = fallback(normal_maps, &self.normal, scene.normal_map());
        let mask_map = fallback(mask_maps, &self.mask, scene.mask_map());

        // And set their uniform values
        uniforms.set_sampler("albedo", albedo_map);
        uniforms.set_sampler("normal", normal_map);
        uniforms.set_sampler("mask", mask_map);
    }
}

impl Standard {
    // Create a new standard builder with default parameters
    pub fn builder(
        ctx: &mut Context,
        assets: &mut Assets,
        shaders: &mut Storage<Shader>,
    ) -> StandardBuilder {
        StandardBuilder(Self {
            albedo: None,
            normal: None,
            mask: None,
            bumpiness: 1.0,
            roughness: 1.0,
            metallic: 0.0,
            tint: vek::Vec3::one(),
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
    pub fn with_tint(mut self, tint: vek::Vec3<f32>) -> Self {
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
