use std::collections::HashMap;

use gui::egui::util::id_type_map::TypeId;
use resources::Resource;

// The world is a container for multiple resources
// All the game engine logic is stored within the world, like ECS and Asset management
// Each World can be created using the builder pattern with the help of an App
pub struct World(HashMap<TypeId, Box<dyn Resource>>);

impl World {
    // Insert a resource into the world
}