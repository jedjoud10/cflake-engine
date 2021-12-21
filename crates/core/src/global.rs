// Global file that contains all world related functions
// Commands grouped for each module
// Entity Component Systems
pub mod ecs {

    use crate::command::*;
    use ecs::stored::*;
    use crate::tasks::*;
    use ecs::Component;
    use lazy_static::lazy_static;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::sync::RwLockReadGuard;

    lazy_static! {
        static ref SYSTEM_COUNTER: AtomicUsize = AtomicUsize::new(0);
    }
    /* #region Entities */
    // Get an entity using it's global ID
    pub fn entity(entity_id: usize) -> Option<ecs::Entity> {
        let w = crate::world::world();
        w.ecs_manager.entitym.entities.get_element(entity_id).flatten().cloned()
    }
    // Entity mut callback. We run this callback at the end of the frame with a world_mut environment 
    pub fn entity_mut(entity: &ecs::Entity, callback: u64) {
        // Create a local callback
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
    pub fn entity_remove(entity: &ecs::Entity) -> CommandQueryResult {
        CommandQueryResult::new(Task::EntityRemove(entity.entity_id))
    }
    /* #endregion */
    /* #region Components */
    // Get a stored component
    fn component_stored<'a, T: Component + 'static>(entity: &'a ecs::Entity) -> Option<Stored<'a, T>> {
        // Get the corresponding global component ID from the entity
        let global_id = entity.linked_components.get(&T::get_component_id())?;
        // Get the world using it's RwLock
        let mut w = crate::world::world();
        let componentm = &w.ecs_manager.componentm;
        let component = componentm.get_component::<T>(*global_id).ok()?;
        Some(Stored::new(component, global_id))
    }
    // Get a stored mutable component
    fn component_stored_mut<'a, T: Component + 'static>(entity: &'a ecs::Entity) -> Option<StoredMut<'a, T>> {
        // Get the corresponding global component ID from the entity
        let global_id = entity.linked_components.get(&T::get_component_id())?;
        // Get the world using it's RwLock
        let mut w = crate::world::world_mut();
        let componentm = &mut w.ecs_manager.componentm;
        let component = componentm.get_component_mut::<T>(*global_id).ok()?;
        Some(StoredMut::new(component, global_id))
    }
    // Get a component
    pub fn component<'a, T: Component + 'static>(entity: &'a ecs::Entity) -> Option<&'a T> {
        let stored = component_stored::<T>(entity)?;
        Some(stored.get(entity))
    }
    // Get a component mutably. However, we can only run this if we are in a EntityMutCallback callback 
    pub fn component_mut<'a, T: Component + 'static>(entity: &'a mut ecs::Entity) -> Option<&'a mut T> {
        let stored = component_stored_mut::<T>(entity)?;
        Some(stored.get_mut(entity))
    }
    /* #endregion */
    /* #region Systems */
    // Add the system on the main thread
    pub fn add_system<T: ecs::CustomSystemData, F>(callback: F)
    where
        F: FnOnce() -> ecs::System<T> + 'static + Send,
    {
        // Create a new thread and initialize the system on it
        SYSTEM_COUNTER.fetch_add(1, Ordering::Relaxed);
        let (join_handle, c_bitfield) = crate::system::create_worker_thread(callback);
        let system_thread_data = ecs::SystemThreadData::new(join_handle, c_bitfield);
        let mut w = crate::world::world_mut();
        w.ecs_manager.systemm.systems.push(system_thread_data);
    }
    // Get the number of valid systems that exist in the world
    pub fn system_counter() -> usize {
        SYSTEM_COUNTER.load(Ordering::Relaxed)
    }  
    /* #endregion */
}
// Input
pub mod input {
    // Bind key
    pub fn bind_key(_key: input::Keys, _map_name: &str, _map_type: input::MapType) {}
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
    pub fn start_keys_reg() {}
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
}
