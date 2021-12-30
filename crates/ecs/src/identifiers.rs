use crate::{Component, component_registry};

// An EntityID that will be used to identify entities
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct EntityID {
    pub index: u16, 
}
impl EntityID { pub fn new(index: u16) -> Self { Self { index } } }

// A ComponentID that will be used to identify components
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct ComponentID {
    pub entity_id: u16,
    pub component_id: u32,
}
impl ComponentID { 
    // Create a new component ID using a component generic and an entity ID
    pub fn new<T: Component>(entity_id: EntityID) -> Self {
        let component_bitfield = component_registry::get_component_bitfield::<T>();
        Self {
            entity_id: entity_id.index,
            component_id: *component_bitfield.bitfield,
        }
    }
}