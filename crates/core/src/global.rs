// Global file that contains all world related functions
// Commands grouped for each module
// Entity Component Systems
pub mod ecs {
    use ecs::{ComponentInternal, Component};

    use crate::command::*;
    // Add an entity without any linking groups
    pub fn entity_add_empty(entity: ecs::Entity) {}
    // Add an entity to the world. Let's hope that this doesn't exceed the maximum theoretical number of entities, which is 18,446,744,073,709,551,615
    pub fn entity_add(entity: ecs::Entity, linkings: ecs::ComponentLinkingGroup) {
        let waitable = command(CommandQuery::single(Task::CreateEntity(entity, linkings)));
        let x = waitable.wait();
    }
    // Remove an entity from the world, returning a WorldCommandStatus of Failed if we failed to do so
    pub fn entity_remove() {}
    // Get a component
    pub fn component<T: Component + 'static>(entity: &ecs::Entity) -> &'static T {
        // Get the corresponding global component ID from the entity
        let global_id = entity.linked_components.get()
        // Get the world using it's RwLock
        let w = crate::world::world();
        w.ecs_manager.componentm.get_component::<T>(global_id)
    }
    // Get a component mutably, since this is going to run at the end of the frame using an FnOnce
    pub fn component_mut<T: Component, F: FnOnce(&mut T)>() {}
    // Create a component linking group
    pub fn component_linking_group(entity: &ecs::Entity) -> ecs::ComponentLinkingGroup {
        ecs::ComponentLinkingGroup::new()
    }
    // This is an alternate function that links the component directly, no linking group
    pub fn link_component_direct() {}
}
// Input
pub mod input {    
    // Bind key 
    pub fn bind_key(key: input::Keys, map_name: &str, _map_type: input::MapType) {}
    // Get the accumulated mouse position
    pub fn mouse_pos() -> (i32, i32) {
        let w = crate::world::world();
        w.input_manager.get_accumulated_mouse_position()
    }
    // Get the accumulated mouse scroll
    pub fn mouse_scroll() -> f32 {
        let w = crate::world::world();
        w.input_manager.get_accumulated_mouse_scroll()
    }
    // Start registering the keys as a sentence
    pub fn start_keys_reg() {
        
    }
    // Check if the key registering is active
    pub fn keys_reg_active() -> bool {
        let w = crate::world::world();
        w.input_manager.keys_reg_active()
    }
    // Stop registering the keys as a sentence and return it
    pub fn stop_keys_reg() -> String { todo!() }
    // Toggle the registering of the keys as a literal string
    pub fn toggle_keys_reg() -> Option<String> { todo!() }
    // Returns true when the map is pressed
    pub fn map_pressed(name: &str) -> bool {
        let w = crate::world::world();
        w.input_manager.map_pressed(name)
    }
    // Returns true when the map is pressed, ignores the update check
    pub fn map_pressed_uncheck(name: &str) -> bool {
        let w = crate::world::world();
        w.input_manager.map_pressed_uncheck(name)
    }
    // Returns true when the map is being held
    pub fn map_held(name: &str) -> (bool, f32) {
        let w = crate::world::world();
        w.input_manager.map_held(name)
    }
    // Returns true when the map has been released
    pub fn map_released(name: &str) -> bool {
        let w = crate::world::world();
        w.input_manager.map_released(name)
    }
    // Returns the toggle state of the map
    pub fn map_toggled(name: &str) -> bool {
        let w = crate::world::world();
        w.input_manager.map_toggled(name)
    }    
}
// User Interface shit
pub mod ui {
    use crate::command::{command, CommandQuery, Task};

    // Add a root the world
    pub fn add_root(name: &str, root: ui::Root) {
        command(CommandQuery::single(Task::AddRoot(name.to_string(), root))).wait();
    }
}
// IO stuff
pub mod io {
    use crate::command::{CommandQuery, Task, command};
    // Create the default config file
    pub fn create_config_file() -> crate::GameConfig {
        command(CommandQuery::single(Task::CreateConfigFile())).wait();
        let w = crate::world::world();
        return w.saver_loader.load::<crate::GameConfig>("config\\game_config.json");
    }
    // Load a copy of the config file
    pub fn load_config_file() -> crate::GameConfig {
        let w = crate::world::world();
        return w.saver_loader.load::<crate::GameConfig>("config\\game_config.json");
    }
    // Load a copy of the saver loader
    pub fn saver_loader() -> io::SaverLoader {
        let w = crate::world::world(); 
        return w.saver_loader;
    }
}
// Mains
pub mod main {
}