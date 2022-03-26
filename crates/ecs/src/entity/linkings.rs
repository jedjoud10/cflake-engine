use crate::prelude::ArchetypeId;

// Entity linking data that we will use to link entities to their specified components
pub struct EntityLinkings {
    // The archetype that the entity is linked to
    pub archetype: ArchetypeId,

    // The index of the components in said archetype
    pub bundle: usize,
}
