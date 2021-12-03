use crate::{Entity, ECSError, system_data::{SystemData, InternalSystemData}, LinkedComponents, ComponentManager, ComponentID, EntityManager};

#[derive(Default)]
// Manages the systems
pub struct SystemManager {
    pub systems: Vec<System>,
}

impl SystemManager {
    // Remove an entity from it's corresponding systems, this is done before actually removing the entity to allow the systems to dispose of it's data
    pub fn remove_entity_from_systems(&mut self, entity: &Entity, entity_id: usize) {
        // Remove the entity from all the systems it was in
        for system in self.systems.iter_mut() {
            // Only remove the entity from the systems that it was in
            if system.is_entity_valid(entity) {
            }
        }
    }
    // Add an entity to it's corresponding systems
    pub fn add_entity_to_systems(&mut self, entity: &Entity) {
        // Check if there are systems that need this entity
        for system in self.systems.iter_mut() {
            if system.is_entity_valid(entity) {
                // Add the entity to the system
            }
        }
    }
    // Add a system to the world, and returns it's system ID
    pub fn add_system(&mut self, mut system: System) -> u8 {
        let id = self.systems.len() as u8;
        system.system_id = id;
        self.systems.push(system);
        id
    }
    // Update all the systems
    pub fn update_systems(&mut self, entity_manager: &EntityManager) {
        for system in self.systems.iter_mut() {
            system.run_system(entity_manager);
        }
    }
    // Kill all the systems
    pub fn kill_systems(&mut self, entity_manager: &EntityManager) {
        for system in self.systems.iter_mut() {
            // Shut down each system first
            system.disable();
        }
        // Then end them
        for system in self.systems.iter_mut() {
            system.end_system(entity_manager);
        }
    }
    // Gets a reference to a system
    pub fn get_system(&self, system_id: u8) -> Result<&System, ECSError> {
        self.systems.get(system_id as usize).ok_or(ECSError::new_str("System does not exist!"))
    }
    // Gets a mutable reference to a system
    pub fn get_system_mut(&mut self, system_id: u8) -> Result<&mut System, ECSError> {
        self.systems.get_mut(system_id as usize).ok_or(ECSError::new_str("System does not exist!"))
    }
    // Gets a reference to the custom data of a specific system
    pub fn get_custom_system_data<U: InternalSystemData + 'static>(&self, system_id: u8) -> Result<&U, ECSError> {
        let system = self.get_system(system_id)?;
        let data = system.system_data.cast::<U>().unwrap();
        return Ok(data);
    }
    // Gets a mutable reference to the custom data a specific system
    pub fn get_custom_system_data_mut<U: InternalSystemData + 'static>(&mut self, system_id: u8) -> Result<&mut U, ECSError> {
        let system = self.get_system_mut(system_id)?;
        let data = system.system_data.cast_mut::<U>().unwrap();
        return Ok(data);
    }
}
// A system event enum
pub enum SystemEventType {
    // Control events
    SystemEnabled(fn(&mut SystemData)),
    SystemDisabled(fn(&mut SystemData)),
    SystemPrefire(fn(&mut SystemData)),
    SystemPostfire(fn(&mut SystemData)),
    // Entity events
    EntityAdded(fn(&mut SystemData, &Entity)),
    EntityRemoved(fn(&mut SystemData, &Entity)),
    EntityUpdate(fn(&mut SystemData, &Entity, &LinkedComponents)),
    // Entity custom event
    EntityFilter(fn(&LinkedComponents) -> bool),
}

// A system, stored on the stack, but it's SystemData is a trait object
#[derive(Default)]
pub struct System {
    required_c_bitfield: usize,
    flc_c_bitfield: usize,
    system_id: u8,
    enabled: bool,
    entities: Vec<usize>,
    // The system data
    system_data: SystemData,

    // Events
    // Control events
    system_enabled_evn: Option<fn(&mut SystemData)>,
    system_disabled_evn: Option<fn(&mut SystemData)>,
    system_prefire_evn: Option<fn(&mut SystemData)>,
    system_postfire_evn: Option<fn(&mut SystemData)>,
    // Entity events
    entity_added_evn: Option<fn(&mut SystemData, &Entity)>,
    entity_removed_evn: Option<fn(&mut SystemData, &Entity)>,
    entity_update_evn: Option<fn(&mut SystemData, &Entity, &LinkedComponents)>,
    entity_filter: Option<fn(&LinkedComponents) -> bool>,
}

// System code
impl System {
    // Check if a specified entity fits the criteria to be in a specific system
    fn is_entity_valid(&self, entity: &Entity) -> bool {
        // Check if the system matches the component ID of the entity
        let bitfield: usize = self.required_c_bitfield & !entity.c_bitfield;
        // If the entity is valid, all the bits would be 0
        bitfield == 0
    }
    // Add a component to this system's component bitfield id
    pub fn link_component<U: ComponentID>(&mut self, component_manager: &mut ComponentManager) -> Result<(), ECSError> {
        if component_manager.is_component_registered::<U>() {
            let c = component_manager.get_component_id::<U>()?;
            self.required_c_bitfield |= c;
            self.flc_c_bitfield |= c;
        } else {
            component_manager.register_component::<U>();
            let c = component_manager.get_component_id::<U>()?;
            self.required_c_bitfield |= c;
            self.flc_c_bitfield |= c;
        }
        Ok(())
    }
    // Add a component that each entity *can* have, this is not necessary
    pub fn link_component_flc_extra<U: ComponentID>(&mut self, component_manager: &mut ComponentManager) -> Result<(), ECSError> {
        if component_manager.is_component_registered::<U>() {
            self.flc_c_bitfield |= component_manager.get_component_id::<U>()?;
        } else {
            component_manager.register_component::<U>();
            self.flc_c_bitfield |= component_manager.get_component_id::<U>()?;
        }
        Ok(())
    }
    // Attach the a specific system event
    pub fn event(&mut self, event: SystemEventType) {
        match event {
            // Control events
            SystemEventType::SystemEnabled(x) => self.system_enabled_evn = Some(x),
            SystemEventType::SystemDisabled(x) => self.system_disabled_evn = Some(x),
            SystemEventType::SystemPrefire(x) => self.system_prefire_evn = Some(x),
            SystemEventType::SystemPostfire(x) => self.system_postfire_evn = Some(x),
            // Entity events
            SystemEventType::EntityAdded(x) => self.entity_added_evn = Some(x),
            SystemEventType::EntityRemoved(x) => self.entity_removed_evn = Some(x),
            SystemEventType::EntityUpdate(x) => self.entity_update_evn = Some(x),
            SystemEventType::EntityFilter(x) => self.entity_filter = Some(x),
        };
    }
    // Add an entity to the current system
    fn add_entity(&mut self, entity: &Entity) {
        if !self.enabled {
            return;
        }
        self.entities.push(entity.entity_id);
        // Fire the event
        match self.entity_added_evn {
            Some(x) => x(&mut self.system_data, entity),
            None => {}
        }
    }
    // Remove an entity from the current system
    fn remove_entity(&mut self, entity_id: usize, entity: &Entity) {
        if !self.enabled {
            return;
        }
        // Search for the entity with the matching entity_id
        let system_entity_local_id = self.entities.iter().position(|&entity_id_in_vec| entity_id_in_vec == entity_id).unwrap();
        self.entities.remove(system_entity_local_id);
        // Fire the event
        match self.entity_removed_evn {
            Some(x) => x(&mut self.system_data, entity),
            None => {}
        }
    }
    // Stop the system permanently
    fn end_system(&mut self, entity_manager: &EntityManager) {
        match self.entity_removed_evn {
            Some(x) => {
                // Fire the entity removed event
                for entity_id in self.entities.iter() {
                    // Get the entity
                    let entity = entity_manager.get_entity(*entity_id).unwrap().clone();
                    x(&mut self.system_data, &entity);
                }
            }
            None => {}
        }
    }
    // Run the system for a single iteration
    fn run_system(&mut self, entity_manager: &EntityManager) {
        if !self.enabled {
            return;
        }
        // Pre fire event
        match self.system_prefire_evn {
            Some(x) => x(&mut self.system_data),
            None => {}
        }

        // Filter the entities
        let entity_manager_immutable = entity_manager;
        let filtered_entity_ids = match self.entity_filter {
            Some(x) => {
                self.entities
                    .iter()
                    .filter(|entity_id| {
                        // Filter the entities
                        let entity_clone = &entity_manager_immutable.get_entity(**entity_id).unwrap();
                        // Get the linked components
                        let linked_components = LinkedComponents::get_linked_components(entity_clone, self.flc_c_bitfield);
                        x(&linked_components)
                    })
                    .cloned()
                    .collect()
            }
            None => {
                // No filtering, just return all the entities
                self.entities.clone()
            }
        };
        // Only update the entities if we have a a valid event. No need to waste time updating them ¯\_(ツ)_/¯
        match self.entity_update_evn {
            Some(x) => {
                // Loop over all the entities and fire the event
                for entity_id in filtered_entity_ids {
                    let entity_clone = entity_manager.get_entity(entity_id).unwrap().clone();
                    // Get the linked entity components from the current entity
                    let linked_components = LinkedComponents::get_linked_components(&entity_clone, self.flc_c_bitfield);
                    x(&mut self.system_data, &entity_clone, &linked_components);
                }
            }
            None => {}
        }

        // Post fire event
        match self.system_postfire_evn {
            Some(x) => x(&mut self.system_data),
            None => {}
        }
    }
    // Enable this system
    pub fn enable(&mut self) {
        self.enabled = true;
        // Fire the event
        match self.system_enabled_evn {
            Some(x) => x(&mut self.system_data),
            None => {}
        }
    }
    // Disable this system
    pub fn disable(&mut self) {
        self.enabled = false;
        /*
        // Fire the event
        match self.system_disabled_evn {
            Some(x) => x(&mut self.system_data, data),
            None => {}
        }
        */
    }
    // With custom data
    pub fn custom_data<U: InternalSystemData + 'static>(&mut self, data: U) {
        self.system_data.convert(data)
    }
}
