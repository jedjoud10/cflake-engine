use time::Time;
use world::{Handle, Read, Storage};

use crate::{
    display::PrimitiveMode,
    material::{AlbedoMap, Material},
    prelude::{FragmentStage, Processor, ShaderCompiler, Uniforms, VertexStage},
    scene::Renderer,
};

use super::DefaultMaterialResources;

// This is the material that our skysphere/skybox will use for rendering
// TODO: Implemented HDRi sky material and sheit
pub struct Sky {
    // Main sky color
    pub gradient: Handle<AlbedoMap>,

    // Sun settings
    pub sun_intensity: f32,
    pub sun_size: f32,
}

impl<'w> Material<'w> for Sky {
    type Resources = (Read<'w, Storage<AlbedoMap>>, Read<'w, Time>);

    fn fetch_resources(world: &'w world::World) -> Self::Resources {
        let maps = world.get::<Storage<AlbedoMap>>().unwrap();
        let time = world.get::<Time>().unwrap();
        (maps, time)
    }

    fn primitive_mode() -> PrimitiveMode {
        PrimitiveMode::Triangles { cull: None }
    }

    fn uses_shadow_pass() -> bool {
        false
    }

    unsafe fn should_assume_valid() -> bool {
        true
    }

    fn should_use_frustum_culling() -> bool {
        false
    }

    fn set_static_properties(
        uniforms: &mut Uniforms,
        main: &DefaultMaterialResources,
        resources: &mut Self::Resources,
    ) {
        uniforms.set_mat4x4("view_matrix", main.camera.view_matrix());
        uniforms.set_mat4x4("proj_matrix", main.camera.projection_matrix());
        uniforms.set_vec3("sun_dir", main.directional_light_rotation.forward());
        uniforms.set_scalar("time_since_startup", resources.1.secs_since_startup_f32());
        uniforms.set_vec3::<vek::Vec3<f32>>("camera", main.camera_location.into());
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
        let texture = resources.0.get(&instance.gradient);
        uniforms.set_sampler("gradient", texture);
        uniforms.set_scalar(
            "sun_intensity",
            main.directional_light.strength * instance.sun_intensity,
        );
        uniforms.set_scalar("sun_size", instance.sun_size);
    }

    fn shader(
        ctx: &mut crate::context::Context,
        assets: &mut assets::Assets,
    ) -> crate::prelude::Shader {
        let vs = assets
            .load::<VertexStage>("engine/shaders/pbr.vrsh.glsl")
            .unwrap();

        let fs = assets
            .load::<FragmentStage>("engine/shaders/sky.frsh.glsl")
            .unwrap();

        ShaderCompiler::link((vs, fs), Processor::new(assets), ctx)
    }
}
