use std::{cell::RefCell, ffi::c_void};

use crate::{cast_component, cast_component_mut, component_registry, Component, ComponentError, ComponentID, ComponentReadGuard, ComponentWriteGuard, EnclosedComponent, EntityID, System, Entity};
use ahash::AHashMap;
use bitfield::Bitfield;
use worker_threads::ThreadPool;
// Some linked components that we can mutate or read from in each system
// These components are stored on the main thread however
pub struct LinkedComponents<> {
    pub(crate) entity: *const Entity,
}

impl LinkedComponents {
    // Create some linked components from an Entity ID, the full AHashMap of components, and the System cbitfield
    // Theoretically, this should only be done once, when an entity becomes valid for a system
    pub(crate) fn new(id: &EntityID, entity: &Entity, components: &AHashMap<ComponentID, RefCell<EnclosedComponent>>, cbitfield: &Bitfield<u32>) -> Self {
        // Get the components from the world, that fit the cbitfield and the Entity ID
        let filtered_components = components
            .iter()
            .filter_map(|(component_id, component)| {
                if cbitfield.contains(&component_id.cbitfield) && component_id.entity_id == *id {
                    // The component is linked to the entity, and we must get the component's pointer
                    let ptr = component.as_ptr();
                    Some((component_id.cbitfield, ptr))
                } else {
                    // The component is not linked to the entity
                    None
                }
            })
            .collect::<AHashMap<Bitfield<u32>, *mut EnclosedComponent>>();
        Self { 
            components: &entity.components as *const _,
            entity_id: id.clone()
        }
    }
}

impl LinkedComponents {
    // Get a reference to a specific linked component
    pub fn component<'b, T>(& self) -> Result<ComponentReadGuard<'b, T>, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {
        // TODO: Make each entity have a specified amount of components so we can have faster indexing using
        // entity_id * 16 + local_component_id
        let id = component_registry::get_component_bitfield::<T>();
        // Kill me
        let hashmap = unsafe { &*self.components };
        let ptr = *hashmap
            .get(&id)
            .ok_or_else(|| ComponentError::new_without_id("Linked component could not be fetched!".to_string()))?;
        // Magic
        let component = unsafe { &*ptr }.as_ref();
        let component = cast_component::<T>(component)?;
        let guard = ComponentReadGuard::new(component);
        Ok(guard)
    }
    // Get a mutable reference to a specific linked entity components struct
    pub fn component_mut<'b, T>(& mut self) -> Result<ComponentWriteGuard<'b, T>, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {
        let id = component_registry::get_component_bitfield::<T>();
        // TODO: Make each entity have a specified amount of components so we can have faster indexing using
        // entity_id * 16 + local_component_id
        // Kill me
        let hashmap = unsafe { &*self.components };
        let ptr = *hashmap
            .get(&id)
            .ok_or_else(|| ComponentError::new_without_id("Linked component could not be fetched!".to_string()))?;
        // Magic
        let component = unsafe { &mut *ptr }.as_mut();
        let component = cast_component_mut::<T>(component)?;
        let guard = ComponentWriteGuard::new(component);
        Ok(guard)
    }
}



// A struct full of LinkedComponents that we send off to update in parallel
// This will use the components data given by the world to run all the component updates in PARALLEL
// The components get mutated in parallel, though the system is NOT stored on another thread
pub struct ComponentQuery<'a> {
    // The actual components
    pub(crate) linked_components: &'a AHashMap<EntityID, LinkedComponents>,
    // A thread pool that is actually stored in the ECS manager
    // This a pointer
    pub(crate) pool: *const c_void,
}

impl<'a> ComponentQuery<'a> {
    // Execute the component query, so we actually update the components
    pub fn update_all<RefContext: 'static>(mut self, context: RefContext, function: fn(&RefContext, &mut LinkedComponents), multithreaded: bool) {
        if !multithreaded {
            // Run it normally
            let elements = self.linked_components.iter_mut();
            for (_, linked_components) in elements {
                function(&context, linked_components);
            }
        } else {
            // Run it using multithreading
            let elements = &mut self.linked_components;
            // Uhhhhh.... magic?
            let pool = unsafe { &*(self.pool as *const ThreadPool<RefContext, LinkedComponents>) };
            let elements = self.linked_components.values_mut().map(|x| x as *mut LinkedComponents).collect::<Vec<_>>();
            pool.execute(elements, &context, function);
        }
    } 
}