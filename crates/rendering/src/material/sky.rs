use ecs::EcsManager;
use math::Transform;
use time::Time;
use world::{Handle, Storage};

use crate::{
    canvas::{Canvas, FaceCullMode},
    context::{Context, Window},
    mesh::Mesh,
    prelude::{FragmentStage, Processor, Shader, ShaderCompiler, Uniforms, VertexStage},
    scene::{Camera, Directional, Renderer, SceneSettings},
};

use super::{AlbedoMap, Material, Pipeline};

// This is the material that our skysphere/skybox will use for rendering
// TODO: Implemented HDRi sky material and sheit
pub struct Sky {
    // Main sky color
    pub gradient: Handle<AlbedoMap>,
    pub offset: f32,

    // Sun settings
    pub sun_intensity: f32,
    pub sun_radius: f32,

    // Cloud settings
    pub cloud_coverage: f32,
    pub cloud_speed: f32,
}

impl<'w> Material<'w> for Sky {
    type Resources = (&'w Storage<AlbedoMap>, &'w Time);

    fn fetch(
        world: &'w mut world::World,
    ) -> (
        &'w SceneSettings,
        &'w EcsManager,
        &'w Storage<Self>,
        &'w Storage<Mesh>,
        &'w mut Storage<Shader>,
        &'w mut Window,
        &'w mut Context,
        Self::Resources,
    ) {
        let (settings, ecs, mats, meshes, shaders, window, context, albedo_maps, time) = world
            .get_mut::<(
                &SceneSettings,
                &EcsManager,
                &Storage<Self>,
                &Storage<Mesh>,
                &mut Storage<Shader>,
                &mut Window,
                &mut Context,
                &Storage<AlbedoMap>,
                &Time,
            )>()
            .unwrap();
        (
            settings,
            ecs,
            mats,
            meshes,
            shaders,
            window,
            context,
            (albedo_maps, time),
        )
    }

    fn face_cull_mode() -> Option<FaceCullMode> {
        Some(FaceCullMode::Front(true))
    }

    // This method will be called once right before we start rendering the batches
    fn set_static_properties<'u>(
        uniforms: &mut Uniforms<'u>,
        resources: &mut Self::Resources,
        _canvas: &Canvas,
        _scene: &SceneSettings,
        camera: (&Camera, &Transform),
        light: (&Directional, &Transform),
    ) where
        'w: 'u,
    {
        uniforms.set_mat4x4("view_matrix", camera.0.view());
        uniforms.set_mat4x4("proj_matrix", camera.0.projection());
        uniforms.set_vec3("sun_dir", light.1.forward());
        uniforms.set_scalar("offset", (light.1.forward().y + 1.0) / 2.0);
        uniforms.set_scalar("time", resources.1.secs_since_startup_f32());
    }

    // This method will be called for each surface that we have to render
    fn set_render_properties<'u>(
        uniforms: &mut Uniforms<'u>,
        _resources: &mut Self::Resources,
        renderer: &Renderer,
        _camera: (&Camera, &Transform),
        _light: (&Directional, &Transform),
    ) where
        'w: 'u,
    {
        uniforms.set_mat4x4("world_matrix", renderer.matrix());
    }

    fn set_instance_properties<'u>(
        &'w self,
        uniforms: &mut Uniforms<'u>,
        resources: &mut Self::Resources,
        _scene: &SceneSettings,
        _camera: (&Camera, &Transform),
        light: (&Directional, &Transform),
    ) where
        'w: 'u,
    {
        let texture = resources.0.get(&self.gradient);
        uniforms.set_sampler("gradient", texture);
        uniforms.set_scalar("sun_intensity", light.0.strength * self.sun_intensity);
        uniforms.set_scalar("sun_radius", self.sun_radius);
        uniforms.set_scalar("cloud_speed", self.cloud_speed);
        uniforms.set_scalar("cloud_coverage", self.cloud_coverage);
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
