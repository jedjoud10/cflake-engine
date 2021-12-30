// Global file that contains all world related functions
// Commands grouped for each module
// Entity Component Systems
pub mod ecs {
    use crate::command::*;
    use crate::tasks::*;
    use ecs::Component;
    use ecs::identifiers::EntityID;
    /* #region Entities */
    // Get an entity using it's global ID
    pub fn entity(id: EntityID) ->  Result<ecs::Entity, ecs::EntityError> {
        let w = crate::world::world();
        w.ecs_manager.entity(id).map(|x| x.clone())
    }
    // Entity mut callback. We run this callback at the end of the frame with a world_mut environment
    pub fn entity_mut(id: EntityID, callback_id: u64) {
        // Create a local callback
        let args = crate::callbacks::LogicSystemCallbackArguments::EntityMut(id);
        crate::callbacks::buffer_callback_execution(callback_id, args);
    }
    // Add an entity without any linking groups
    pub fn entity_add_empty(entity: ecs::Entity) -> CommandQueryResult {
        let empty_linkings = ecs::ComponentLinkingGroup::new();
        entity_add(entity, empty_linkings)
    }
    // Add an entity to the world. Let's hope that this doesn't exceed the maximum theoretical number of entities, which is 18,446,744,073,709,551,615
    pub fn entity_add(entity: ecs::Entity, linkings: ecs::ComponentLinkingGroup) -> CommandQueryResult {
        CommandQueryResult::new(Task::EntityAdd(entity, linkings))
    }
    // Remove an entity from the world, returning a WorldCommandStatus of Failed if we failed to do so
    pub fn entity_remove(id: EntityID) -> CommandQueryResult {
        CommandQueryResult::new(Task::EntityRemove(id))
    }
    /* #endregion */
    /* #region Components */
    // Get a component
    pub fn component<'a, T: Component + 'static>(entity: &'a ecs::Entity) -> Result<&'a T, ecs::ComponentError> {
        // Get the world using it's RwLock
        let w = crate::world::world();
        // Create a component ID
        let id = ecs::ComponentID::new(entity.id, ecs::registry::get_component_bitfield::<T>());
        let component = w.ecs_manager.component::<T>(id)?;
        let component_ = component as *const T;
        let component = unsafe { &*component_ };
        Ok(component)
    }
    // Get a component mutably. However, we can only run this if we are in a EntityMutCallback callback
    pub fn component_mut<'a, T: Component + 'static>(entity: &'a mut ecs::Entity) -> Result<&'a mut T, ecs::ComponentError> {
        // Get the world using it's RwLock
        let mut w = crate::world::world_mut();
        // Create a component ID
        let id = ecs::ComponentID::new(entity.id, ecs::registry::get_component_bitfield::<T>());
        let component = w.ecs_manager.component_mut::<T>(id)?;
        let component_ = component as *mut T;
        let component = unsafe { &mut *component_ };
        Ok(component)
    }
    // Manually add a component linking group to an already existing entity
    // If some components collide, we will panic
    pub fn link_components(id: EntityID, linkings: ecs::ComponentLinkingGroup) -> CommandQueryResult {
        CommandQueryResult::new(Task::AddComponentLinkingGroup(id, linkings))
    }
    /* #endregion */
    /* #region Systems */
    // Add the system on the main thread
    pub fn add_system<T: ecs::CustomSystemData + 'static, F>(default_state: T, callback: F)
    where
        F: FnOnce() -> ecs::System<T> + 'static + Send,
        T: Sync + Send,
    {
        // Create a new thread and initialize the system on it
        let (join_handle, cbitfield) = crate::system::create_worker_thread(default_state, callback);
        let mut w = crate::world::world_mut();
        // Calculate the system bitfield
        let system_thread_data = ecs::SystemThreadData::new(join_handle, cbitfield);
        w.ecs_manager.add_system(system_thread_data);
    }
    /* #endregion */
}
// Input
pub mod input {
    // Bind key
    pub fn bind_key(key: input::Keys, map_name: &str, map_type: input::MapType) {
        let mut w = crate::world::world_mut();
        w.input_manager.bind_key(key, map_name, map_type);
    }
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
        let mut w = crate::world::world_mut();
        w.input_manager.start_keys_reg();
    }
    // Check if the key registering is active
    pub fn keys_reg_active() -> bool {
        let w = crate::world::world();
        w.input_manager.keys_reg_active()
    }
    // Stop registering the keys as a sentence and return it
    pub fn stop_keys_reg() -> String {
        todo!()
    }
    // Toggle the registering of the keys as a literal string
    pub fn toggle_keys_reg() -> Option<String> {
        todo!()
    }
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

    // Add a root the world
    pub fn add_root(_name: &str, _root: ui::Root) {
        /*
        let mut w = crate::world::world_mut();
        command(CommandQuery::new(Task::AddRoot(name.to_string(), root)));
        */
    }
}
// IO stuff
pub mod io {
    // Create the default config file
    pub fn create_config_file() -> crate::GameConfig {
        let mut w = crate::world::world_mut();
        let saver_loader = &mut w.saver_loader;
        saver_loader.create_default("config\\game_config.json", &crate::GameConfig::default());
        // Then load
        saver_loader.load("config\\game_config.json")
    }
    // Load a copy of the config file
    pub fn load_config_file() -> crate::GameConfig {
        let w = crate::world::world();
        w.saver_loader.load::<crate::GameConfig>("config\\game_config.json")
    }
}
// Mains
pub mod main {
    use lazy_static::lazy_static;
    use std::sync::Arc;

    use crate::{communication::RECEIVER, custom_world_data::CustomWorldData};
    // Get the world custom data
    pub fn world_data() -> CustomWorldData {
        let w = crate::world::world();
        w.custom_data.clone()
    }
    pub fn world_data_mut<F: FnOnce(&mut CustomWorldData)>(f: F) {
        let mut w = crate::world::world_mut();
        let custom_data = &mut w.custom_data;
        (f)(custom_data);
    }
    // Send a message to all the Logic Systems, telling them to start their loops and to clear their starting buffer
    pub fn start_system_loops() {
        let receiver_ = RECEIVER.lock().unwrap();
        let receiver = receiver_.as_ref().unwrap();
        crate::system::send_lsc_all(crate::system::LogicSystemCommand::StartSystemLoop, receiver);
    }
}
// Callback shit
pub mod callbacks {
    pub use crate::callbacks::*;
}
// Timings
pub mod timings {
    // Get the delta time
    pub fn delta() -> f64 {
        let w = crate::world::world();
        w.time_manager.delta_time
    }
    // Get the elapsed time since the start of the game
    pub fn elapsed() -> f64 {
        let w = crate::world::world();
        w.time_manager.elapsed
    }
    // Get the frame count
    pub fn frame_count() -> u64 {
        let w = crate::world::world();
        w.time_manager.frame_count
    }
}

// Batches
pub mod batch {
    use crate::{batch::BatchCommandQuery, command::CommandQueryResult};

    // Create a new batch command query for a specific ID and add a specific command onto it
    pub fn batch_add(batch_id: u32, command_result: CommandQueryResult) {
        crate::system::batch_add(batch_id, command_result);
    }
    // Send a specific thread local batch to the main thread
    // The delete parameters tells us if we should delete the batch after sending it or just clearing it
    pub fn send_batch(batch_id: u32, delete: bool) {
        crate::system::send_batch(batch_id, delete);
    }
}
