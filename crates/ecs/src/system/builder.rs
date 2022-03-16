use crate::{component::{ComponentQueryParams, ComponentQuerySet}, event::RcEvents};

use super::{SubSystem, System, SystemExecutionOrder, SystemSet};

// A system builder used to build multiple systems
pub struct SystemBuilder<'a, World> {
    set: &'a mut SystemSet,
    events: RcEvents<World>,
    system: System,
}

impl<'a, World> SystemBuilder<'a, World> {
    // Create a new system builder
    pub(crate) fn new(set: &'a mut SystemSet, events: RcEvents<World>) -> Self {
        Self {
            set,
            system: System {
                subsystems: Default::default(),
                evn_index: Default::default(),
                order: SystemExecutionOrder::default(),
            },
            events
        }
    }
    // Set the system's execution order
    pub fn order(mut self, order: SystemExecutionOrder) -> Self {
        self.system.order = order;
        self
    }
    // Add a subsystem with specific component query parameters
    pub fn query(mut self, params: ComponentQueryParams) -> Self {
        self.system.subsystems.push(SubSystem {
            cbitfield: params.cbitfield,
            all: Default::default(),
            delta: Default::default(),
        });
        self
    }
    // Set the "Run System" event of this system
    pub fn event(mut self, evn: fn(&mut World, ComponentQuerySet)) -> Self {
        let mut borrowed = self.events.borrow_mut();
        let index = borrowed.len();
        borrowed.push(evn);
        self.system.evn_index = Some(index);
        drop(borrowed);
        self
    }
    // Build this system and add it to the ECS manager
    pub fn build(self) {
        self.set.add(self.system)
    }
}
