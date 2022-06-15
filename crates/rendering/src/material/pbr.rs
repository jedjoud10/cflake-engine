use assets::Assets;
use ecs::EcsManager;
use world::resources::{Handle, Storage};

use crate::{
    context::{Context, Graphics},
    mesh::SubMesh,
    scene::SceneRenderer,
    shader::{FragmentStage, Processor, Shader, ShaderCompiler, Uniforms, VertexStage},
    texture::{Ranged, Texture, Texture2D, RG, RGB, RGBA},
};

use super::{
    BatchRenderer, InstanceID, Material, MaterialBuilder, MaterialRenderer, PropertyBlock,
};

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

    // Current instance
    instance: InstanceID<Self>,
}

impl Material for Standard {
    type Renderer = BatchRenderer<Self>;

    fn default(id: InstanceID<Self>) -> Self {
        Self {
            albedo: None,
            normal: None,
            mask: None,
            bumpiness: 1.0,
            roughness: 1.0,
            metallic: 0.0,
            instance: id,
        }
    }

    fn instance(&self) -> &InstanceID<Self> {
        &self.instance
    }

    fn renderer(
        ctx: &mut Context,
        loader: &mut Assets,
        storage: &mut Storage<Shader>,
    ) -> Self::Renderer {
        // Load the vertex shader stage
        let vs = loader
            .load::<VertexStage>("engine/shaders/pbr.vrsh.glsl")
            .unwrap();

        // Load the fragment shader stage
        let fs = loader
            .load::<FragmentStage>("engine/shaders/pbr.frsh.glsl")
            .unwrap();

        // Link the two stages and compile the shader
        let shader = ShaderCompiler::link((vs, fs), Processor::new(loader), ctx);

        // Cache the shader (even though it's unique)
        let handle = storage.insert(shader);

        // Create the batch renderer from this shader handle
        BatchRenderer::from(handle)
    }
}

impl MaterialBuilder<Standard> {
    // Set the albedo map
    pub fn albedo(mut self, albedo: &Handle<AlbedoMap>) -> Self {
        self.material_mut().albedo = Some(albedo.clone());
        self
    }

    // Set the normal map
    pub fn normal(mut self, normal: &Handle<NormalMap>) -> Self {
        self.material_mut().normal = Some(normal.clone());
        self
    }

    // Set the mask map
    pub fn mask(mut self, mask: &Handle<MaskMap>) -> Self {
        self.material_mut().mask = Some(mask.clone());
        self
    }

    // Set the bumpiness parameter
    pub fn bumpiness(mut self, bumpiness: f32) -> Self {
        self.material_mut().bumpiness = bumpiness;
        self
    }

    // Set the roughness parameter
    pub fn roughness(mut self, roughness: f32) -> Self {
        self.material_mut().roughness = roughness;
        self
    }

    // Set the metallic parameter
    pub fn metallic(mut self, metallic: f32) -> Self {
        self.material_mut().metallic = metallic;
        self
    }
}

impl<'world> PropertyBlock<'world> for Standard {
    type PropertyBlockResources = (
        &'world SceneRenderer,
        &'world Storage<AlbedoMap>,
        &'world Storage<NormalMap>,
        &'world Storage<MaskMap>,
    );

    fn set_instance_properties(
        &'world self,
        uniforms: &mut Uniforms,
        resources: &Self::PropertyBlockResources,
    ) {
        // Decompose the fetched resource references
        let (renderer, albedo_maps, normal_maps, mask_maps) = resources;

        // Fallback to the given handle if the first handle is missing
        fn fallback<'a, T: 'static>(
            storage: &'a Storage<T>,
            opt: &Option<Handle<T>>,
            fallback: &Handle<T>,
        ) -> &'a T {
            opt.as_ref()
                .map(|handle| storage.get(handle))
                .unwrap_or_else(|| storage.get(fallback))
        }

        // Scalar parameters
        uniforms.set_scalar("_bumpiness", self.bumpiness);
        uniforms.set_scalar("_roughness", self.roughness);
        uniforms.set_scalar("_metallic", self.metallic);

        // Try to fetch the textures
        let albedo_map = fallback(albedo_maps, &self.albedo, &renderer.albedo_map());
        let normal_map = fallback(normal_maps, &self.normal, &renderer.normal_map());
        let mask_map = fallback(mask_maps, &self.mask, &renderer.mask_map());

        // Get their corresponding samplers
        let albedo_map_sampler = Texture::sampler(albedo_map);
        let normal_map_sampler = Texture::sampler(normal_map);
        let mask_map_sampler = Texture::sampler(mask_map);

        // And set their uniform values
        uniforms.set_sampler("_albedo", albedo_map_sampler);
        uniforms.set_sampler("_normal", normal_map_sampler);
        uniforms.set_sampler("_mask", mask_map_sampler);
    }

    fn fetch(
        world: &'world mut world::World,
    ) -> (
        &'world EcsManager,
        &'world Storage<Self>,
        &'world Storage<SubMesh>,
        &'world mut Storage<Shader>,
        &'world mut Graphics,
        Self::PropertyBlockResources,
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
            scene_renderer,
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
                &SceneRenderer,
            )>()
            .unwrap();
        (
            ecs_manager,
            materials,
            submesh,
            shaders,
            graphics,
            (scene_renderer, albedo_maps, normal_maps, mask_maps),
        )
    }
}

impl MaterialRenderer for BatchRenderer<Standard> {
    fn render(
        &self,
        world: &mut world::World,
        settings: &crate::scene::SceneRenderer,
    ) -> Option<super::Stats> {
        self.render_batched_surfaces(world, settings)
    }
}
