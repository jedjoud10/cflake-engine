use assets::Assets;
use ecs::Scene;
use graphics::{Graphics};
use utils::Storage;
use world::{user, System, World};

use crate::{
    Environment, DeferredRenderer, Pipelines, Renderer, Surface,
};

// Add the envinronment resource into the world and the sky entity
fn init(world: &mut World) {
    // Add the sky entity
    let graphics = world.get::<Graphics>().unwrap();
    let assets = world.get::<Assets>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let _renderer = world.get::<DeferredRenderer>().unwrap();

    // Create the environment resource that contains the cubemaps
    let environment = Environment::new(&graphics, &assets);

    // Drop fetched resources
    drop(graphics);
    drop(assets);
    drop(scene);
    drop(_renderer);

    // Ajoute la resource dans le monde
    world.insert(environment);
}

// Render a single face of the environment map each frame
// Swap the envmap index when done
fn render(_world: &mut World) {
    // TODO: Pls fix texture mip level layer shit (it shit)
    /*
    let mut _environment = world.get_mut::<Environment>().unwrap();
    let environment = &mut *_environment;
    let graphics = world.get::<Graphics>().unwrap();
    let mut pass = ComputePass::begin(&graphics);
    let mut active = pass.bind_shader(&environment.shader);
    let cubemap = &mut environment.temp;
    active.set_bind_group(0, |group| {
        group.set_storage_texture_mut("enviro", cubemap).unwrap()
    }).unwrap();
    active.dispatch(vek::Vec3::one()).unwrap();

    graphics.submit(false);
    let map = &mut environment.environment_map[0];
    let mips = &cubemap.mips();
    let input = mips.level(0).unwrap();
    let mips_mut = &map.mips_mut();
    let mut level = mips_mut.level_mut(0).unwrap();
    level.copy_subregion_from::<TempEnvironmentMap>(input, None, None).unwrap();
    */
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
