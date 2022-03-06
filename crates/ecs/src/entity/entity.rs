use crate::component::{Component, ComponentKey};
use ahash::AHashMap;
use bitfield::Bitfield;
use getset::Getters;
use slotmap::Key;
// A simple entity in the world
#[derive(Getters)]
pub struct Entity {
    // This entity's Key
    #[getset(get = "pub")]
    pub(crate) key: EntityKey,

    // Component Bitfield
    #[getset(get = "pub")]
    pub(crate) cbitfield: Bitfield<u32>,

    // Our stored components
    #[getset(get = "pub")]
    pub(crate) components: AHashMap<Bitfield<u32>, ComponentKey>,
}

// ECS time bois
impl Default for Entity {
    // Create a new default entity
    fn default() -> Self {
        Self {
            key: EntityKey::null(),
            cbitfield: Bitfield::default(),
            components: AHashMap::new(),
        }
    }
}

impl Entity {
    // Check if we have a component linked onto this entity
    pub fn is_component_linked<T: Component + 'static>(&self) -> bool {
        // Get the cbitfield of the component
        let cbitfield = crate::component::registry::get_component_bitfield::<T>();
        self.cbitfield.contains(&cbitfield)
    }
}

slotmap::new_key_type! {
    pub struct EntityKey;
}
