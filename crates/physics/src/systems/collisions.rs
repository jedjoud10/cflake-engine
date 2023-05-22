use world::{System, World};

// Handle collision detection every tick
fn tick(_world: &mut World) {}

// Create the collision system
pub fn system(system: &mut System) {
    system.insert_tick(tick);
}
