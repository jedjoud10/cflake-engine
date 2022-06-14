use world::World;

use crate::{Assets, asset, persistent};


// This system will pre-load all the default assets that we will need
pub fn pre_load_defaults(world: &mut World) {
    let loader = world.get_mut::<&mut Assets>().unwrap();
    
    // Load the default shaders
    persistent!(loader, "engine/shaders/pbr.vrsh.glsl");
    persistent!(loader, "engine/shaders/pbr.frsh.glsl");
}