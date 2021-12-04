// Global file that contains all world related functions
// Commands grouped for each module
// Entity Component Systems
pub mod ecs {
    // Add an entity without any linking groups
    pub fn entity_add_empty(entity: ecs::Entity) {}
    // Add an entity to the world. Let's hope that this doesn't exceed the maximum theoretical number of entities, which is 18,446,744,073,709,551,615
    pub fn entity_add(entity: ecs::Entity, linkings: ecs::ComponentLinkingGroup) {}
    // Remove an entity from the world, returning a WorldCommandStatus of Failed if we failed to do so
    pub fn entity_remove() {}
    // Get a component
    pub fn component() {}
    // Get a component mutably
    pub fn component_mut() {}
    // Create a component linking group
    pub fn component_linking_group(entity: &ecs::Entity) -> ecs::ComponentLinkingGroup {
        ecs::ComponentLinkingGroup::new(entity)
    }
    // This is an alternate function that links the component directly, no linking group
    pub fn link_component_direct() {}
}
// Input
pub mod input {
    // Create the key cache at the start of the world initialization
    pub fn create_key_cache() {}
    // Bind key 
    pub fn bind_key(key: input::Keys, map_name: &str, _map_type: input::MapType) {}
    // Get the accumulated mouse position
    pub fn mouse_pos() -> (i32, i32) {}
    // Get the accumulated mouse scroll
    pub fn mouse_scroll() -> f32 {}
    // Start registering the keys as a sentence
    pub fn start_keys_reg() {}
    // Check if the key registering is active
    pub fn keys_reg_active() -> bool {}
    // Stop registering the keys as a sentence and return it
    pub fn stop_keys_reg() -> String {}
    // Toggle the registering of the keys as a literal string
    pub fn toggle_keys_reg() -> Option<String> {}
    // Returns true when the map is pressed
    pub fn map_pressed(name: &str) -> bool {}
    // Returns true when the map is pressed, ignores the update check
    pub fn map_pressed_uncheck(name: &str) -> bool {}
    // Returns true when the map is being held
    pub fn map_held(name: &str) -> (bool, f32) {}
    // Returns true when the map has been released
    pub fn map_released(name: &str) -> bool {}
    // Returns the toggle state of the map
    pub fn map_toggled(name: &str) -> bool {}    
}
// User Interface shit
pub mod ui {

}
// Mains
pub mod main {

}