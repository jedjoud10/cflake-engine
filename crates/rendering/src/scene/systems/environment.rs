use assets::Assets;
use ecs::Scene;
use graphics::Graphics;
use utils::Storage;
use world::{World, System, user};

use crate::{SkyMaterial, ForwardRenderer, Pipelines, Surface, Renderer, Environment};

// Add the envinronment resource into the world and the sky entity
fn init(world: &mut World) {
    // Add the sky entity
    let graphics = world.get::<Graphics>().unwrap();
    let assets = world.get::<Assets>().unwrap();
    let mut skies = world.get_mut::<Storage<SkyMaterial>>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let _renderer = world.get::<ForwardRenderer>().unwrap();
    let mut pipelines = world.get_mut::<Pipelines>().unwrap();

    // Get the material id (also registers the material pipeline)
    let id = pipelines
        .register::<SkyMaterial>(&graphics, &assets)
        .unwrap();
    let material = skies.insert(SkyMaterial {});
    let mesh = _renderer.sphere.clone();

    // Create the new sky entity components
    let surface = Surface::new(mesh, material, id);
    let renderer = Renderer::default();
    scene.insert((surface, renderer));
    
    // Create the environment resource that contains the cubemaps
    let environment = Environment::new(&graphics, &assets);

    // Drop fetched resources
    drop(graphics);
    drop(assets);
    drop(skies);
    drop(pipelines);
    drop(scene);
    drop(_renderer);

    // Ajoute la resource dans le monde
    world.insert(environment);
}

// Render a single face of the environment map each frame
// Swap the envmap index when done
fn render(world: &mut World) {
}

// The environment system is responsible for creatin the HDRi environment map to use for specular and diffuse IBL
pub fn system(system: &mut System) {
    system.insert_init(init)
        .before(user)
        .after(assets::system)
        .after(graphics::common)
        .after(crate::systems::rendering::system);
}
