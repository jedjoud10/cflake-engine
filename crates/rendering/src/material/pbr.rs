use std::marker::PhantomData;

use ecs::EcsManager;
use world::resources::{Handle, Storage};

use crate::{
    context::{Context, Graphics},
    shader::{FragmentStage, Matrix, Processor, Shader, ShaderCompiler, Uniforms, VertexStage},
    texture::{Ranged, Sampler, Texture, Texture2D, R, RGB},
};

use super::{Material, PropertyBlock, InstanceID, InstanceBuilder, BatchRenderer};

// Type aliases for texture maps
type DiffuseMap = Texture2D<RGB<Ranged<u8>>>;
type NormalMap = Texture2D<RGB<Ranged<u8>>>;
type MaskMap = Texture2D<RGB<Ranged<u8>>>;

// A standard Physically Based Rendering material that we will use by default
// PBR Materials try to replicate the behavior of real light for better graphical fidelty and quality
pub struct Standard {
    // Texture maps used for rendering
    albedo: Option<Handle<DiffuseMap>>,
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
    fn default(id: InstanceID<Self>) -> Self {
        Self { albedo: None, normal: None, mask: None, bumpiness: 1.0, roughness: 1.0, metallic: 0.0, instance: id }
    }

    fn instance(&self) -> &InstanceID<Self> {
        &self.instance
    }

    fn load_shader(
        ctx: &mut Context,
        loader: &mut assets::loader::AssetLoader,
    ) -> crate::shader::Shader {
        let vs = loader
            .load::<VertexStage>("defaults/shaders/rendering/pbr.vrsh")
            .unwrap();
        let fs = loader
            .load::<FragmentStage>("defaults/shaders/rendering/pbr.frsh")
            .unwrap();
        ShaderCompiler::link((vs, fs), Processor::new(loader), ctx)
    }

    type Render = BatchRenderer<Self>;

    fn renderer() -> Self::Render {
        todo!()
        //BatchRenderer::default()
    }
}

impl InstanceBuilder<Standard> {
    // Set the albedo texture
    pub fn albedo(mut self, albedo: Handle<DiffuseMap>) -> Self {
        self.material_mut().albedo = Some(albedo);
        self
    }

    // Set the normal texture
    pub fn normal(mut self, normal: Handle<NormalMap>) -> Self {
        self.material_mut().normal = Some(normal);
        self
    }

    // Set the mask texture
    pub fn mask(mut self, mask: Handle<MaskMap>) -> Self {
        self.material_mut().mask = Some(mask);
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

impl<'a> PropertyBlock<'a> for Standard {
    type Resources = (
        &'a Storage<DiffuseMap>,
        &'a Storage<NormalMap>,
        &'a Storage<MaskMap>,
    );

    fn set_uniforms(&self, uniforms: &mut Uniforms, res: Self::Resources) {
        // Scalar parameters
        uniforms.set_scalar("_bumpiness", self.bumpiness);
        uniforms.set_scalar("_roughness", self.roughness);
        uniforms.set_scalar("_metallic", self.metallic);

        // Try to fetch the textures
        let albedo_map = res.0.try_get(self.albedo.as_ref()).unwrap();
        let normal_map = res.1.try_get(self.normal.as_ref()).unwrap();
        let mask_map = res.2.try_get(self.mask.as_ref()).unwrap();

        // Get their corresponding samplers
        let albedo_map_sampler = Texture::sampler(albedo_map);
        let normal_map_sampler = Texture::sampler(normal_map);
        let mask_map_sampler = Texture::sampler(mask_map);

        // And set their uniform values
        uniforms.set_sampler("_albedo", albedo_map_sampler);
        uniforms.set_sampler("_normal", normal_map_sampler);
        uniforms.set_sampler("_mask", mask_map_sampler);
    }

    fn fetch_resources(
        set: &'a mut world::World,
    ) -> (
        &'a EcsManager,
        &'a mut Graphics,
        &'a Storage<Self>,
        &'a Storage<Shader>,
        Self::Resources,
    ) {
        let (ecs, graphics, materials, shaders, albedo_maps, normal_maps, mask_maps) = set
            .get_mut::<(
                &EcsManager,
                &mut Graphics,
                &Storage<Standard>,
                &Storage<Shader>,
                &Storage<DiffuseMap>,
                &Storage<NormalMap>,
                &Storage<MaskMap>,
            )>()
            .unwrap();
        (
            ecs,
            graphics,
            materials,
            shaders,
            (albedo_maps, normal_maps, mask_maps),
        )
    }
}
/*
impl MaterialBuilder<Standard> {
    
}

impl MaterialRenderer for RenderDescriptor<Standard> {
    fn shader(&self) -> &Handle<Shader> {
        &self.shader
    }

    fn render(&self, resources: &mut world::resources::ResourceSet) {
        let (ecs, graphics, storage) = resources.get_mut::<(&EcsManager, &Graphics, &Storage<Standard>)>().unwrap();
    }
}
*/