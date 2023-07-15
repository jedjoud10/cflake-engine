use crate::Mask;
use slotmap::{new_key_type, KeyData};


new_key_type! {
    /// Entities could be though of as a simple handle to some underlying data stored within the world scene.
    /// Entities by themselves do not store any data, and they can be represented using a [u64] integer handle. 
    pub struct Entity;
}

impl Entity {
    /// Convert the entity handle to a raw [u64].
    pub fn to_raw(self) -> u64 {
        self.0.as_ffi()
    }

    /// Convert a raw [u64] to an entity handle.
    pub fn from_raw(raw: u64) -> Self {
        Self(KeyData::from_ffi(raw))
    }
}

/// Entity linking data that we will use to link entities to their specified components
#[derive(Clone, Copy)]
pub struct EntityLinkings {
    pub(crate) mask: Mask,
    pub(crate) index: usize,
}

impl EntityLinkings {
    /// Get the mask of the entity (the mask of it's current archetype).
    pub fn mask(&self) -> Mask {
        self.mask
    }

    /// Get the index of the entity in it's current archetype.
    pub fn index(&self) -> usize {
        self.index
    }
}
