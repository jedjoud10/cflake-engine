// Global file that contains all world related functions
// Commands grouped for each module
// Entity Component Systems
pub mod ecs {
    use crate::callbacks::CallbackSendingData;
    use crate::callbacks::CallbackType;
    use crate::callbacks::RefCallback;
    use crate::command::*;
    use crate::tasks::*;
    use ecs::Component;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::sync::{RwLockReadGuard, RwLockWriteGuard};
    use lazy_static::lazy_static;
    lazy_static! {
        static ref SYSTEM_COUNTER: AtomicUsize = AtomicUsize::new(0);
    }    
    /* #region Entities */
    // Get an entity using it's global ID
    pub fn entity(entity_id: usize) -> Option<ecs::Entity> {
        let w = crate::world::world();
        w.ecs_manager.entitym.entities.get_element(entity_id).unwrap().cloned()
    }
    // Add an entity without any linking groups
    pub fn entity_add_empty(entity: ecs::Entity) {
        let empty_linkings = ecs::ComponentLinkingGroup::empty();
        entity_add(entity, empty_linkings)
    }
    // Add an entity to the world. Let's hope that this doesn't exceed the maximum theoretical number of entities, which is 18,446,744,073,709,551,615
    pub fn entity_add(entity: ecs::Entity, linkings: ecs::ComponentLinkingGroup) {
        command(CommandQuery::new(Task::EntityAdd(entity, linkings, CallbackSendingData::None)))
    }
    // Callback counter part
    pub fn entity_add_callback<F: Fn(&ecs::Entity) + 'static>(entity: ecs::Entity, linkings: ecs::ComponentLinkingGroup, callback: F) {
        let boxed_callback = Box::new(callback);
        let ref_callback = RefCallback::new(boxed_callback);
        command(CommandQuery::new(Task::EntityAdd(entity, linkings, CallbackSendingData::ValidCallback(0))))
    }
    // Remove an entity from the world, returning a WorldCommandStatus of Failed if we failed to do so
    pub fn entity_remove(entity: &ecs::Entity) {
        command(CommandQuery::new(Task::EntityRemove(entity.entity_id)))
    }
    /* #endregion */
    /* #region Components */
    // Get a component
    pub fn component<'a, T: Component + 'static>(entity: &ecs::Entity) -> ecs::stored::Stored<T> {
        // Get the corresponding global component ID from the entity
        let global_id = entity.linked_components.get(&T::get_component_id()).unwrap();
        // Get the world using it's RwLock
        let w: RwLockReadGuard<'static, crate::world::World> = crate::world::world();
        let componentm = &w.ecs_manager.componentm;
        componentm.get_component::<T>(*global_id).unwrap()
    }
    // Get a component mutably, since this is going to run at the end of the frame using an FnOnce
    pub fn component_mut<T: Component + 'static, F: Fn(&'static mut T)>(entity: &ecs::Entity, _callback: F) {
        /* #region We are on the main thread */
        let main_thread = crate::system::IS_MAIN_THREAD.with(|x| x.get());
        if main_thread {
            // We are on the main thread, we can get the world as a mut
            let mut world: RwLockWriteGuard<'static, crate::world::World> = crate::world::world_mut();
            // Get the corresponding global component ID from the entity
            let global_id = entity.linked_components.get(&T::get_component_id()).unwrap();
            let componentm = &mut world.ecs_manager.componentm;
            let _x = componentm.get_component_mut::<T>(*global_id).unwrap();
            //callback(x);
        }
        /* #endregion */
        else {
            // At the end of the current frame, run the callback on the main thread (If we are on a worker thread)

        }
    }
    // Create a component linking group
    pub fn component_linking_group(entity: &ecs::Entity) -> ecs::ComponentLinkingGroup {
        ecs::ComponentLinkingGroup::new(entity)
    }
    // This is an alternate function that links the component directly, no linking group
    pub fn link_component_direct() {}
    /* #endregion */
    /* #region Systems */
    // Add the system on the main thread
    pub fn add_system<T: ecs::CustomSystemData, F>(callback: F)
    where
        F: FnOnce() -> ecs::System<T> + 'static + Send,
    {
        // Create a new thread and initialize the system on it
        SYSTEM_COUNTER.fetch_add(1, Ordering::Relaxed);
        let join_handle = crate::system::create_worker_thread(callback);
        let system_thread_data = ecs::SystemThreadData::new(join_handle);
        let mut w = crate::world::world_mut();
        w.ecs_manager.systemm.systems.push(system_thread_data);
    }
    // Get the number of valid systems that exist in the world
    pub fn system_counter() -> usize { SYSTEM_COUNTER.load(Ordering::Relaxed) }
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
    use crate::command::{command, CommandQuery};
    use crate::tasks::Task;

    // Add a root the world
    pub fn add_root(name: &str, root: ui::Root) {
        command(CommandQuery::new(Task::AddRoot(name.to_string(), root)));
    }
}
// IO stuff
pub mod io {
    use crate::command::{command, CommandQuery};
    use crate::tasks::Task;

    // Create the default config file
    pub fn create_config_file() -> crate::GameConfig {
        let mut w = crate::world::world_mut();
        let saver_loader = &mut w.saver_loader;
        saver_loader.create_default("config\\game_config.json", &crate::GameConfig::default());
        // Then load
        return saver_loader.load("config\\game_config.json");
    }
    // Load a copy of the config file
    pub fn load_config_file() -> crate::GameConfig {
        let w = crate::world::world();
        w.saver_loader.load::<crate::GameConfig>("config\\game_config.json")
    }
}
// Mains
pub mod main {
    use others::WorldBarrierData;
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}, Barrier, BarrierWaitResult, RwLock, RwLockReadGuard};
    use lazy_static::lazy_static;
    lazy_static! {
        static ref BARRIERS_WORLD: Arc<WorldBarrierData> = Arc::new(WorldBarrierData::new_uninit());
    }
    // Initialize the world barrier data with the specified amount of threads to wait for
    pub fn init(n: usize) {
        let x = BARRIERS_WORLD.as_ref();
        x.new_update(n);
    }
    // As ref
    pub fn as_ref() -> &'static WorldBarrierData {
        BARRIERS_WORLD.as_ref()
    }
    // Clone
    pub fn clone() -> Arc<WorldBarrierData> { BARRIERS_WORLD.clone() }
}
