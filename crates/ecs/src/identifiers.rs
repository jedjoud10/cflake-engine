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
    pub component_id: u16,
}