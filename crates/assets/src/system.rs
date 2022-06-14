use world::World;

use crate::{Assets, asset};


// This system will pre-load all the default assets that we will need
pub fn pre_load_defaults(world: &mut World) {
    let loader = world.get_mut::<&mut Assets>().unwrap();
    asset!(loader, "./assets/defaults/test.txt");
}