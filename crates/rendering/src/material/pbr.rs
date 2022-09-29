use assets::Assets;

use ecs::Scene;
use itertools::Itertools;
use world::{Handle, Read, Storage};

use crate::{
    context::Context,
    display::Display,
    mesh::{EnabledAttributes, Surface},
    prelude::{CubeMap2D, TextureImportSettings, R, RGBA, SRGBA},
    scene::{ClusteredShading, Renderer, ShadowMapping},
    shader::{FragmentStage, Processor, Shader, ShaderCompiler, Uniforms, VertexStage},
    texture::{Ranged, Texture, Texture2D, RGB},
};

use super::{DefaultMaterialResources, Material, Sky, HDRI};

// PBR texels
pub type AlbedoTexel = SRGBA<Ranged<u8>>;
pub type NormalTexel = RGB<Ranged<u8>>;
pub type MaskTexel = RGBA<Ranged<u8>>;

// PBR maps
pub type AlbedoMap = Texture2D<AlbedoTexel>;
pub type NormalMap = Texture2D<NormalTexel>;
pub type MaskMap = Texture2D<MaskTexel>; // (r = AO, g = roughness, b = metallic)

// A standard Physically Based Rendering material that we will use by default
// PBR Materials try to replicate the behavior of real light for better graphical fidelty and quality
pub struct Standard {
    pub albedo_map: Option<Handle<AlbedoMap>>,
    pub normal_map: Option<Handle<NormalMap>>,
    pub mask_map: Option<Handle<MaskMap>>,
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
        Read<'w, Storage<HDRI>>,
        Read<'w, ShadowMapping>,
        Handle<HDRI>,
    );

    fn requirements() -> EnabledAttributes {
        EnabledAttributes::POSITIONS
            | EnabledAttributes::NORMALS
            | EnabledAttributes::TANGENTS
            | EnabledAttributes::TEX_COORDS
    }

    unsafe fn should_assume_valid() -> bool {
        true
    }

    fn fetch_resources(world: &'w world::World) -> Self::Resources {
        let albedo_map = world.get::<Storage<AlbedoMap>>().unwrap();
        let hdris = world.get::<Storage<HDRI>>().unwrap();
        let normal_map = world.get::<Storage<NormalMap>>().unwrap();
        let mask_map = world.get::<Storage<MaskMap>>().unwrap();
        let shadow_mapping = world.get::<ShadowMapping>().unwrap();

        let ecs = world.get::<Scene>().unwrap();
        let entity = world
            .get::<ClusteredShading>()
            .unwrap()
            .skysphere_entity
            .unwrap();
        let entity = ecs.entry(entity).unwrap();
        let component = entity.get::<Surface<Sky>>().unwrap();
        let sky_materials = world.get::<Storage<Sky>>().unwrap();
        let material = sky_materials.get(&component.material);

        (
            albedo_map,
            normal_map,
            mask_map,
            hdris,
            shadow_mapping,
            material.irradiance.clone(),
        )
    }

    fn set_static_properties<'u>(
        uniforms: &mut Uniforms<'u>,
        main: &DefaultMaterialResources,
        resources: &mut Self::Resources,
    ) {
        uniforms.set_mat4x4("view_matrix", main.camera.view_matrix());
        uniforms.set_mat4x4("proj_matrix", main.camera.projection_matrix());
        uniforms.set_vec3::<vek::Vec3<f32>>("camera_position", main.camera_location.into());
        uniforms.set_vec3("camera_forward", main.camera_rotation.forward());
        uniforms.set_vec3("sun_dir", main.directional_light_rotation.forward());
        uniforms.set_vec3(
            "sun_color",
            main.directional_light.color.as_::<f32>() / 255.0,
        );
        uniforms.set_scalar("sun_strength", main.directional_light.strength);
        let shadow = &(*resources.4);
        uniforms.set_sampler("shadow_map", &shadow.depth_tex);
        uniforms.set_mat4x4(
            "shadow_lightspace_matrix",
            shadow.proj_matrix * shadow.view_matrix,
        );
        uniforms.set_vec2(
            "resolution",
            vek::Vec2::<u32>::from(main.window.viewport().extent.as_::<u32>()),
        );
        uniforms.set_scalar("cluster_size", main.cluster_size);
        uniforms.set_scalar("point_lights_num", main.point_lights.len() as u32);
        uniforms.set_shader_storage_buffer("point_lights", &main.point_lights);
    }

    fn set_surface_properties(
        uniforms: &mut Uniforms,
        _main: &DefaultMaterialResources,
        _resources: &mut Self::Resources,
        renderer: &Renderer,
    ) {
        uniforms.set_mat4x4("world_matrix", &renderer.matrix);
    }

    fn set_instance_properties(
        uniforms: &mut Uniforms,
        main: &DefaultMaterialResources,
        resources: &mut Self::Resources,
        instance: &Self,
    ) {
        let (albedo_maps, normal_maps, mask_maps, hdris, _, irradiance) = resources;

        uniforms.set_vec3("tint", instance.tint);
        uniforms.set_scalar("bumpiness", instance.bumpiness);
        uniforms.set_scalar("roughness", instance.roughness);
        uniforms.set_scalar("metallic", instance.metallic);
        uniforms.set_scalar("ambient_occlusion", instance.ambient_occlusion);
        uniforms.set_vec2("scale", instance.scale);

        let albedo_map = instance
            .albedo_map
            .as_ref()
            .map_or(main.white, |h| albedo_maps.get(h));
        let normal_map = instance
            .normal_map
            .as_ref()
            .map_or(main.normal, |h| normal_maps.get(h));
        let mask_map = instance
            .mask_map
            .as_ref()
            .map_or(main.mask, |h| mask_maps.get(h));
        let irradiance_map = hdris.get(&irradiance);

        uniforms.set_sampler("albedo", albedo_map);
        uniforms.set_sampler("normal", normal_map);
        uniforms.set_sampler("mask", mask_map);
        uniforms.set_sampler("irradiance", irradiance_map);
    }

    fn shader(ctx: &mut Context, assets: &mut Assets) -> Shader {
        let vs = assets
            .load::<VertexStage>("engine/shaders/scene/pbr/pbr.vrtx.glsl")
            .unwrap();

        let fs = assets
            .load::<FragmentStage>("engine/shaders/scene/pbr/pbr.frag.glsl")
            .unwrap();

        ShaderCompiler::link((vs, fs), Processor::new(assets), ctx)
    }
}

// Convert 3 separate ambient occlusion, roughness, and metallic textures into one ARM mask texture
pub fn combine_into_mask(
    ctx: &mut Context,
    ambient_occlusion: Texture2D<R<Ranged<u8>>>,
    roughness: Texture2D<R<Ranged<u8>>>,
    metallic: Texture2D<R<Ranged<u8>>>,
    settings: TextureImportSettings,
) -> Option<MaskMap> {
    // Check if all the textures have the same size
    let resolution = ambient_occlusion.dimensions();
    if resolution != roughness.dimensions() || resolution != metallic.dimensions() {
        return None;
    }

    // Get the first mip level of each texture
    let ao = ambient_occlusion.mip(0).unwrap().download();
    let roughness = roughness.mip(0).unwrap().download();
    let metallic = metallic.mip(0).unwrap().download();

    // Combine all the texels into one vector
    let texels = (0..(resolution.as_::<u32>().product()))
        .into_iter()
        .map(|i| {
            vek::Vec4::new(
                ao[i as usize],
                roughness[i as usize],
                metallic[i as usize],
                0,
            )
        })
        .collect::<Vec<_>>();
    MaskMap::new(
        ctx,
        settings.mode,
        resolution,
        settings.sampling,
        settings.mipmaps,
        Some(&texels),
    )
}
