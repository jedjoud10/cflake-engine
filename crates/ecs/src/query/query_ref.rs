use utils::bitset::BitSet;

use std::{iter::FusedIterator, marker::PhantomData};

use crate::{
    archetype::Archetype,
    layout::{LayoutAccess, QueryLayoutRef},
    scene::Scene,
};

use super::{Always, QueryFilter, Wrap, len};

/// This is a query that will be fetched from the main scene that we can use to get components out of entries with a specific layout.
/// Even though I define the 'it, 'b, and 's lifetime, I don't use them in this query, I only use them in the query iterator.
pub struct QueryRef<'a, L: QueryLayoutRef<'a>> {
    pub(super) archetypes: Vec<&'a Archetype>,
    pub(super) access: LayoutAccess,
    //pub(super) bitsets: Option<Vec<BitSet<u64>>>,
    _phantom3: PhantomData<L>,
}

impl<'a, L: QueryLayoutRef<'a>> QueryRef<'a, L> {
    // Create a new mut query from the scene for active entities
    pub(crate) fn new(scene: &'a Scene) -> Self {
        let (mask, archetypes, _) = super::archetypes::<L, Always>(scene.archetypes());
        Self {
            archetypes,
            //bitsets: None,
            _phantom3: PhantomData,
            access: mask,
        }
    }

    /*
    // Create a new mut query from the scene, but make it have a specific entry enable/disable masks
    pub(crate) fn new_with_filter<F: QueryFilter>(
        scene: &'a Scene,
        _: Wrap<F>,
        ticked: bool,
    ) -> Self {
        // Filter out the archetypes then create the bitsets
        let (access, archetypes, cached) = super::archetypes::<L, F>(scene.archetypes());
        let bitsets =
            super::generate_bitset_chunks::<F>(archetypes.iter().map(|a| &**a), cached, ticked);

        Self {
            archetypes,
            access,
            bitsets: Some(bitsets),
            _phantom3: PhantomData,
        }
    }
    */

    /// Get the access masks that we have calculated.
    pub fn layout_access(&self) -> LayoutAccess {
        self.access
    }

    /// Get the number of entries that we will have to iterate through.
    pub fn len(&self) -> usize {
        len(&self.archetypes)
    }

    /// Check if the query is empty.
    pub fn is_empty(&self) -> bool {
        self.archetypes.is_empty()
    }
}