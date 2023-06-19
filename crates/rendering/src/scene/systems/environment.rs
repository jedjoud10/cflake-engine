use assets::Assets;
use ecs::Scene;
use graphics::{Graphics, ComputePass, ActivePipeline, Texture, GpuPod};
use utils::Storage;
use world::{user, System, World};

use crate::{
    Environment, DeferredRenderer, Pipelines, Renderer, Surface, DirectionalLight,
};

// Add the envinronment resource into the world and the sky entity
fn init(world: &mut World) {
    // Add the sky entity
    let graphics = world.get::<Graphics>().unwrap();
    let assets = world.get::<Assets>().unwrap();
    
    // Create the environment resource that contains the cubemaps
    let environment = Environment::new(&graphics, &assets, 512);

    // Drop fetched resources
    drop(graphics);
    drop(assets);
    
    // Ajoute la resource dans le monde
    world.insert(environment);
}

// Render a single face of the environment map each frame
// Swap the envmap index when done
fn render(world: &mut World) {
    let mut _environment = world.get_mut::<Environment>().unwrap();
    let environment = &mut *_environment;
    let graphics = world.get::<Graphics>().unwrap();
    let renderer = world.get::<DeferredRenderer>().unwrap();
    let scene = world.get::<Scene>().unwrap();

    // Skip if we don't have a light to draw with
    let Some(directional_light)  = renderer.main_directional_light else {
        return;
    };

    // Get the directioanl light and rotation of the light
    let directional_light = scene.entry(directional_light).unwrap();
    let (_, &directional_light_rotation) = directional_light
        .as_query::<(&DirectionalLight, &coords::Rotation)>()
        .unwrap();
    let rotation = directional_light_rotation.forward();

    // Create a new compute shader pass
    let mut pass = ComputePass::begin(&graphics);

    // Get the base environment map to set its view
    let cubemap = &mut environment.environment_map;
    let view = cubemap.view_mut(1).unwrap();
    
    // Generate the base environment map
    let resolution = environment.resolution;
    let matrices = &environment.matrices;
    let mut active = pass.bind_shader(&environment.environment_shader);
    active.set_bind_group(0, |group| {
        group.set_uniform_buffer("matrices", matrices, ..).unwrap();
        group.set_storage_texture_mut("enviro", view).unwrap();
    }).unwrap();
    active.set_push_constants(|pc| {
        let bytes = rotation.into_bytes();
        pc.push(bytes, 0).unwrap();
    }).unwrap();
    active.dispatch(vek::Vec3::new(resolution / 32, resolution / 32, 6)).unwrap();

    // Generate the diffuse IBL map
    let resolution = environment.resolution / 16;
    let src_cubemap = &environment.environment_map;
    let dst_cubemap = &mut environment.diffuse_ibl_map;
    let view = dst_cubemap.view_mut(1).unwrap();
    let mut active = pass.bind_shader(&environment.ibl_diffuse_convolution_shader);
    active.set_bind_group(0, |group| {
        group.set_uniform_buffer("matrices", matrices, ..).unwrap();
        group.set_sampled_texture("enviro", src_cubemap).unwrap();
        group.set_sampler("enviro_sampler", src_cubemap.sampler().unwrap()).unwrap();
        group.set_storage_texture_mut("diffuse", view).unwrap();
    }).unwrap();
    active.dispatch(vek::Vec3::new(resolution / 32, resolution / 32, 6)).unwrap();

    graphics.submit(false);
}

// The environment system is responsible for creatin the HDRi environment map to use for specular and diffuse IBL
pub fn system(system: &mut System) {
    system
        .insert_init(init)
        .before(user)
        .after(assets::system)
        .after(graphics::common)
        .after(crate::systems::rendering::system);

    system
        .insert_update(render)
        .before(user)
        .before(crate::systems::rendering::system);
}
