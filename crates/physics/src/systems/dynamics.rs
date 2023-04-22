use crate::Velocity;
use coords::Position;
use ecs::Scene;
use utils::Time;
use world::{post_user, System, World};

// Update the Rigidbodies in the world
fn tick(world: &mut World) {
    // Integrate velocity to position
    let mut scene = world.get_mut::<Scene>().unwrap();
    let time = world.get::<Time>().unwrap();
    for (velocity, position) in scene.query_mut::<(&Velocity, &mut Position)>() {
        **position += **velocity * time.tick_delta().as_secs_f32() * time.tick_interpolation();
    }
}

// Create the dynamics system
pub fn system(system: &mut System) {
    system
        .insert_tick(tick)
        .after(post_user)
        .after(crate::systems::collisions::system);
}
