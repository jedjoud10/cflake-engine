use assets::Assets;

use world::{Handle, Read, Storage};

use crate::{
    context::Context,
    mesh::EnabledAttributes,
    prelude::{RGBA, SRGBA},
    scene::Renderer,
    shader::{FragmentStage, Processor, Shader, ShaderCompiler, Uniforms, VertexStage},
    texture::{Ranged, Texture, Texture2D, RG, RGB},
};

use super::{DefaultMaterialResources, Material};

// PBR maps
pub type AlbedoMap = Texture2D<SRGBA<Ranged<u8>>>;
pub type NormalMap = Texture2D<RGB<Ranged<u8>>>;
pub type MaskMap = Texture2D<RG<Ranged<u8>>>; // (r = roughness, g = metallic)

// A standard Physically Based Rendering material that we will use by default
// PBR Materials try to replicate the behavior of real light for better graphical fidelty and quality
pub struct Standard {
    pub albedo_map: Handle<AlbedoMap>,
    pub normal_map: Handle<NormalMap>,
    pub mask_map: Handle<MaskMap>,
    pub bumpiness: f32,
    pub roughness: f32,
    pub metallic: f32,
    pub tint: vek::Rgb<f32>,
}

impl<'w> Material<'w> for Standard {
    type Resources = (
        Read<'w, Storage<AlbedoMap>>,
        Read<'w, Storage<NormalMap>>,
        Read<'w, Storage<MaskMap>>,
    );

    fn requirements() -> EnabledAttributes {
        EnabledAttributes::POSITIONS
            | EnabledAttributes::NORMALS
            | EnabledAttributes::TANGENTS
            | EnabledAttributes::TEX_COORDS
    }

    fn primitive_mode() -> crate::display::PrimitiveMode {
        crate::display::PrimitiveMode::Triangles { cull: None }
    }

    fn fetch_resources(world: &'w world::World) -> Self::Resources {
        let albedo_map = world.get::<Storage<AlbedoMap>>().unwrap();
        let normal_map = world.get::<Storage<NormalMap>>().unwrap();
        let mask_map = world.get::<Storage<MaskMap>>().unwrap();
        (albedo_map, normal_map, mask_map)
    }

    fn set_static_properties<'u>(
        uniforms: &mut Uniforms<'u>,
        main: &DefaultMaterialResources,
        _resources: &mut Self::Resources,
    ) {
        uniforms.set_mat4x4("view_matrix", main.camera.view_matrix());
        uniforms.set_mat4x4("proj_matrix", main.camera.projection_matrix());
        uniforms.set_vec3::<vek::Vec3<f32>>("camera", main.camera_location.into());
        uniforms.set_vec3("forward", main.camera_rotation.forward());
        uniforms.set_vec3("light_dir", main.directional_light_rotation.forward());
        uniforms.set_scalar("light_strength", main.directional_light.strength);
    }

    fn set_surface_properties(
        uniforms: &mut Uniforms,
        _main: &DefaultMaterialResources,
        _resources: &mut Self::Resources,
        renderer: &Renderer,
    ) {
        uniforms.set_mat4x4("world_matrix", renderer.matrix());
    }

    fn set_instance_properties(
        uniforms: &mut Uniforms,
        _main: &DefaultMaterialResources,
        resources: &mut Self::Resources,
        instance: &Self,
    ) {
        let (albedo_maps, normal_maps, mask_maps) = resources;

        uniforms.set_vec3("tint", instance.tint);
        uniforms.set_scalar("bumpiness", instance.bumpiness);
        uniforms.set_scalar("roughness", instance.roughness);
        uniforms.set_scalar("metallic", instance.metallic);

        let albedo_map = albedo_maps.get(&instance.albedo_map);
        let normal_map = normal_maps.get(&instance.normal_map);
        let mask_map = mask_maps.get(&instance.mask_map);

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
