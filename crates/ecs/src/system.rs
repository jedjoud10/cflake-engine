use crate::{Component, Entity, linked_components::{LinkedComponents, LinkedComponentsID}, ComponentID, EnclosedComponent};
use ahash::AHashMap;
use bitfield::Bitfield;

// A system event enum
pub enum SystemEventType
{
    // Component events
    UpdateComponents(fn(&mut LinkedComponents)),
}

// A system that updates specific components in parallel
pub struct System
{
    cbitfield: Bitfield<u32>, // Our Component Bitfield
    // Events
    update_components: Option<fn(&mut LinkedComponents)>,
    // The component IDs
    linked_components_ids: Vec<LinkedComponentsID>,
}

// Initialization of the system
impl System {
    // Create a new system
    pub fn new() -> Self {
        System {
            cbitfield: Bitfield::<u32>::default(),
            update_components: None,
            linked_components_ids: Vec::new(),
        }
    }
}

// System code
impl System {
    // Add a component to this system's component bitfield id
    pub fn link<U: Component>(&mut self) {
        let c = crate::registry::get_component_bitfield::<U>();
        self.cbitfield = self.cbitfield.add(&c);
    }
    // Attach the a specific system event
    pub fn event(&mut self, event: SystemEventType) {
        match event {
            // Component events
            SystemEventType::UpdateComponents(x) => self.update_components = Some(x),
        };
    }
    // Run the system for a single iteration
    // This will use the components data given by the world to run all the component updates in PARALLEL
    // The components get mutated in parallel, though the system is NOT in parallel
    pub fn run_system(&self, components: &mut AHashMap<ComponentID, EnclosedComponent>)
    {
        // These components are filtered for us
        if let Some(evn) = self.update_components {
            let components = self.linked_components_ids.iter().map(|id| LinkedComponents::new(id, components, &self.cbitfield));
            // This can be ran in parallel, and yet still be totally safe
            for mut linked_components in components {
                evn(&mut linked_components);
            }
        }
    }
}