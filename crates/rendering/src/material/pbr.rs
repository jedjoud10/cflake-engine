use assets::Assets;

use ecs::{Scene, Entity};
use world::{Handle, Read, Storage};

use crate::{
    context::Context,
    mesh::{EnabledAttributes, Surface},
    prelude::{TextureImportSettings, R, RGBA, SRGBA},
    scene::{Renderer, Compositor, ClusteredShading},
    shader::{FragmentStage, Processor, Shader, ShaderCompiler, Uniforms, VertexStage},
    texture::{Ranged, Texture, Texture2D, RG, RGB},
};

use super::{DefaultMaterialResources, Material, Sky};

// PBR maps
pub type AlbedoMap = Texture2D<SRGBA<Ranged<u8>>>;
pub type NormalMap = Texture2D<RGB<Ranged<u8>>>;
pub type MaskMap = Texture2D<RGBA<Ranged<u8>>>; // (r = roughness, g = metallic, b = AO)

// A standard Physically Based Rendering material that we will use by default
// PBR Materials try to replicate the behavior of real light for better graphical fidelty and quality
pub struct Standard {
    pub albedo_map: Handle<AlbedoMap>,
    pub normal_map: Handle<NormalMap>,
    pub mask_map: Handle<MaskMap>,
    pub bumpiness: f32,
    pub roughness: f32,
    pub ambient_occlusion: f32,
    pub metallic: f32,
    pub tint: vek::Rgb<f32>,
    pub scale: vek::Vec2<f32>,
}

impl<'w> Material<'w> for Standard {
    type Resources = (
        Read<'w, Storage<AlbedoMap>>,
        Read<'w, Storage<NormalMap>>,
        Read<'w, Storage<MaskMap>>,
        Handle<AlbedoMap>
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

        let ecs = world.get::<Scene>().unwrap();
        let entity = world.get::<ClusteredShading>().unwrap().skysphere_entity.unwrap();
        let entity = ecs.entry(entity).unwrap();
        let component = entity.get::<Surface<Sky>>().unwrap();
        let sky_materials = world.get::<Storage<Sky>>().unwrap();
        let material = sky_materials.get(&component.material());

        (albedo_map, normal_map, mask_map, material.gradient.clone())
    }

    fn set_static_properties<'u>(
        uniforms: &mut Uniforms<'u>,
        main: &DefaultMaterialResources,
        resources: &mut Self::Resources,
    ) {
        uniforms.set_mat4x4("view_matrix", main.camera.view_matrix());
        uniforms.set_mat4x4("proj_matrix", main.camera.projection_matrix());
        uniforms.set_vec3::<vek::Vec3<f32>>("camera", main.camera_location.into());
        uniforms.set_vec3("forward", main.camera_rotation.forward());
        uniforms.set_vec3("light_dir", main.directional_light_rotation.forward());
        uniforms.set_vec3(
            "light_color",
            main.directional_light.color.as_::<f32>() / 255.0,
        );
        uniforms.set_scalar("light_strength", main.directional_light.strength);

        
        uniforms.set_sampler("gradient", resources.0.get(&resources.3));
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
        let (albedo_maps, normal_maps, mask_maps, _) = resources;

        uniforms.set_vec3("tint", instance.tint);
        uniforms.set_scalar("bumpiness", instance.bumpiness);
        uniforms.set_scalar("roughness", instance.roughness);
        uniforms.set_scalar("metallic", instance.metallic);
        uniforms.set_scalar("ambient_occlusion", instance.ambient_occlusion);
        uniforms.set_vec2("scale", instance.scale);

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

// Create a new mask map using the separate metallic, roughness, and AO parameters
// Roughness is stored in the Red channel
// Metallic is stored in the Green channel
// AO is stored in the Blue channel
pub fn combine_into_mask_map(
    ctx: &mut Context,
    roughness: Texture2D<R<Ranged<u8>>>,
    metallic: Texture2D<R<Ranged<u8>>>,
    ao: Texture2D<R<Ranged<u8>>>,
    settings: TextureImportSettings,
) -> Option<MaskMap> {
    let dimensions = roughness.dimensions();
    if metallic.dimensions() != dimensions || ao.dimensions() != dimensions {
        return None;
    }

    // This is extremely slow but I don't know how to copy specific channels between texture images
    // Also don't want to use FBO as well lol
    let roughness_mip = roughness.mip(0).unwrap();
    let roughness_data = roughness_mip.download();
    let metallic_mip = metallic.mip(0).unwrap();
    let metallic_data = metallic_mip.download();
    let ambient_mip = ao.mip(0).unwrap();
    let ambient_data = ambient_mip.download();

    // I hate this so much
    let data = (0..roughness.texel_count())
        .map(|i| {
            let index = i as usize;
            let r = roughness_data[index];
            let g = metallic_data[index];
            let b = ambient_data[index];
            vek::Vec4::new(r, g, b, 0u8)
        })
        .collect::<Vec<_>>();

    Some(
        MaskMap::new(
            ctx,
            settings.mode,
            dimensions,
            settings.sampling,
            settings.mipmaps,
            Some(&data),
        )
        .unwrap(),
    )
}
