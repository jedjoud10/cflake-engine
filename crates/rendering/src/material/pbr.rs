use assets::Assets;
use ecs::EcsManager;
use math::Transform;
use world::{Handle, Storage, World};

use crate::{
    context::{Context, Graphics, Device},
    mesh::{SubMesh, Surface},
    scene::{SceneSettings, Camera, Renderer},
    shader::{FragmentStage, Processor, Shader, ShaderCompiler, Uniforms, VertexStage},
    texture::{Ranged, Texture, Texture2D, RG, RGB, RGBA}, canvas::Canvas,
};

use super::{
    InstanceID, Material, PropertyBlock, Pipeline, batch_renderer, SinglePassPipeline,
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

    // Unique parameters
    tint: vek::Vec3<f32>,

    // Current instance
    instance: InstanceID<Self>,
}

impl From<InstanceID<Standard>> for Standard {
    fn from(id: InstanceID<Standard>) -> Self {
        Self {
            albedo: None,
            normal: None,
            mask: None,
            bumpiness: 1.0,
            roughness: 1.0,
            metallic: 0.0,
            tint: vek::Vec3::one(),
            instance: id,
        }
    }
}

impl Material for Standard {
    fn instance(&self) -> &InstanceID<Self> {
        &self.instance
    }

    fn shader(ctx: &mut Context, loader: &mut Assets) -> Shader {
        let vs = loader
            .load::<VertexStage>("engine/shaders/pbr.vrsh.glsl")
            .unwrap();

        let fs = loader
            .load::<FragmentStage>("engine/shaders/pbr.frsh.glsl")
            .unwrap();

        ShaderCompiler::link((vs, fs), Processor::new(loader), ctx)
    }

    type Pipe = SinglePassPipeline<Self>;
}

/*
        
        
        */


/*
// Set the albedo map
    pub fn with_albedo(mut self, albedo: &Handle<AlbedoMap>) -> Self {
        self.material_mut().albedo = Some(albedo.clone());
        self
    }

    // Set the normal map
    pub fn with_normal(mut self, normal: &Handle<NormalMap>) -> Self {
        self.material_mut().normal = Some(normal.clone());
        self
    }

    // Set the mask map
    pub fn with_mask(mut self, mask: &Handle<MaskMap>) -> Self {
        self.material_mut().mask = Some(mask.clone());
        self
    }

    // Set the tint parameter
    pub fn with_tint(mut self, tint: vek::Vec3<f32>) -> Self {
        self.material_mut().tint = tint;
        self
    }

    // Set the bumpiness parameter
    pub fn with_bumpiness(mut self, bumpiness: f32) -> Self {
        self.material_mut().bumpiness = bumpiness;
        self
    }

    // Set the roughness parameter
    pub fn with_roughness(mut self, roughness: f32) -> Self {
        self.material_mut().roughness = roughness;
        self
    }

    // Set the metallic parameter
    pub fn with_metallic(mut self, metallic: f32) -> Self {
        self.material_mut().metallic = metallic;
        self
    }
*/

impl<'world> PropertyBlock<'world> for Standard {
    type PropertyBlockResources = (
        &'world Storage<AlbedoMap>,
        &'world Storage<NormalMap>,
        &'world Storage<MaskMap>,
    );

    // This method will be called once right before we start rendering the batches
    fn set_static_properties<'u>(
        uniforms: &mut Uniforms<'u>,
        resources: &Self::PropertyBlockResources,
        canvas: &Canvas,
        scene: &SceneSettings,
        camera: (&Camera, &Transform),
    ) where 
        'world: 'u {
        uniforms.set_mat4x4("view_matrix", camera.0.view());
        uniforms.set_mat4x4("proj_matrix", camera.0.projection());
        uniforms.set_vec3("camera", camera.1.position);
        uniforms.set_vec3("forward", camera.1.forward());
    }

    // This method will be called for each surface that we have to render
    fn set_render_properties<'u>(
        uniforms: &mut Uniforms<'u>,
        resources: &Self::PropertyBlockResources,
        renderer: &Renderer,
        camera: (&Camera, &Transform),
    ) where 
        'world: 'u {
        uniforms.set_mat4x4("world_matrix", renderer.matrix());
    }

    // This method will be called whenever we detect a material instance change
    fn set_instance_properties<'u>(
        &'world self,
        uniforms: &mut Uniforms<'u>,
        resources: &Self::PropertyBlockResources,
        scene: &SceneSettings,
        camera: (&Camera, &Transform),
    ) where
        'world: 'u,
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

    fn fetch(
        world: &'world mut world::World,
    ) -> (
        &'world SceneSettings,
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
}
