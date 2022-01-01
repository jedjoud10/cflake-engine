// Global file that contains all world related functions
// Commands grouped for each module
// Entity Component Systems
pub mod ecs {
    use ecs::identifiers::EntityID;
    use ecs::{Component};

    use crate::world::ECS_MANAGER;
    /* #region Entities */
    // Get an entity using it's global ID
    pub fn entity<'a>(id: &'a EntityID) -> Result<&'a ecs::Entity, ecs::EntityError> {
        let ecs = ECS_MANAGER.borrow_mut();
        let entity = ecs.entity(*id)?;
        // This is totally safe, since we do not mutate ANYTHING about the world (The singular thing that we do mutate are components) during the frame
        let entity = unsafe { &*(entity as *const ecs::Entity) };
        Ok(entity)
    }
    // Add an entity to the world.
    pub fn entity_add(entity: ecs::Entity, linkings: ecs::ComponentLinkingGroup) -> EntityID {
        //
    }
    // Remove an entity from the world, returning a WorldCommandStatus of Failed if we failed to do so
    pub fn entity_remove(id: EntityID) {
        //
    }
    /* #endregion */
    /* #region Components */
    // Get a component
    pub(crate) fn component<'a, T: Component + 'static>(entity: &'a ecs::Entity) -> Result<&'a T, ecs::ComponentError> {
        // Get the world using it's RwLock
        let ecs = ECS_MANAGER.borrow();
        // Create a component ID
        let id = ecs::ComponentID::new(entity.id, ecs::registry::get_component_bitfield::<T>());
        let component = ecs.component::<T>(id)?;
        let component_ = component as *const T;
        let component = unsafe { &*component_ };
        Ok(component)
    }
    // Get a component mutably. However, we can only run this if we are in a EntityMutCallback callback
    pub(crate) fn component_mut<'a, T: Component + 'static>(entity: &'a mut ecs::Entity) -> Result<&'a mut T, ecs::ComponentError> {
        // Get the world using it's RwLock
        let mut ecs = ECS_MANAGER.borrow_mut();
        // Create a component ID
        let id = ecs::ComponentID::new(entity.id, ecs::registry::get_component_bitfield::<T>());
        let component = ecs.component_mut::<T>(id)?;
        let component_ = component as *mut T;
        let component = unsafe { &mut *component_ };
        Ok(component)
    }
    // Manually add a component linking group to an already existing entity
    // If some components collide, we will panic
    pub fn link_components(id: EntityID, linkings: ecs::ComponentLinkingGroup) {
        //
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
    use crate::world::INPUT_MANAGER;

    // Create the key cache
    pub fn create_key_cache() {
        let mut input = INPUT_MANAGER.borrow_mut();
        input.create_key_cache();    
    }
    // Bind key
    pub fn bind_key(key: input::Keys, map_name: &str, map_type: input::MapType) {
        let mut input = INPUT_MANAGER.borrow_mut();
        input.bind_key(key, map_name, map_type);
    }
    // Get the accumulated mouse position
    pub fn mouse_pos() -> (i32, i32) {
        let input = INPUT_MANAGER.borrow();
        input.get_accumulated_mouse_position()
    }
    // Get the accumulated mouse scroll
    pub fn mouse_scroll() -> f32 {
        let input = INPUT_MANAGER.borrow();
        input.get_accumulated_mouse_scroll()
    }
    // Start registering the keys as a sentence
    pub fn start_keys_reg() {
        let mut input = INPUT_MANAGER.borrow_mut();
        input.start_keys_reg();
    }
    // Check if the key registering is active
    pub fn keys_reg_active() -> bool {
        let input = INPUT_MANAGER.borrow();
        input.keys_reg_active()
    }
    // Stop registering the keys as a sentence and return it
    pub fn stop_keys_reg() -> String {
        let mut input = INPUT_MANAGER.borrow_mut();
        input.stop_keys_reg()
    }
    // Toggle the registering of the keys as a literal string
    pub fn toggle_keys_reg() -> Option<String> {
        todo!()
    }
    // Returns true when the map is pressed
    pub fn map_pressed(name: &str) -> bool {
        let input = INPUT_MANAGER.borrow();
        input.map_pressed(name)
    }
    // Returns true when the map is pressed, ignores the update check
    pub fn map_pressed_uncheck(name: &str) -> bool {
        let input = INPUT_MANAGER.borrow();
        input.map_pressed_uncheck(name)
    }
    // Returns true when the map is being held
    pub fn map_held(name: &str) -> (bool, f32) {
        let input = INPUT_MANAGER.borrow();
        input.map_held(name)
    }
    // Returns true when the map has been released
    pub fn map_released(name: &str) -> bool {
        let input = INPUT_MANAGER.borrow();
        input.map_released(name)
    }
    // Returns the toggle state of the map
    pub fn map_toggled(name: &str) -> bool {
        let input = INPUT_MANAGER.borrow();
        input.map_toggled(name)
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
    use crate::world::IO;

    // Create the default config file
    pub fn create_config_file() -> crate::GameConfig {
        let saver_loader = IO.borrow_mut();
        saver_loader.create_default("config\\game_config.json", &crate::GameConfig::default());
        // Then load
        saver_loader.load("config\\game_config.json")
    }
    // Load a copy of the config file
    pub fn load_config_file() -> crate::GameConfig {
        let saver_loader = IO.borrow();
        saver_loader.load::<crate::GameConfig>("config\\game_config.json")
    }
}
// Timings
pub mod timings {
    use crate::{context::RefContext, world::TIME};

    // Get the delta time
    pub fn delta() -> f64 {        
        TIME.borrow().time_manager.delta_time
    }
    // Get the elapsed time since the start of the game
    pub fn elapsed() -> f64 {
        TIME.borrow().time_manager.elapsed
    }
    // Get the frame count
    pub fn frame_count() -> u64 {
        TIME.borrow().time_manager.frame_count
    }
}