use std::{iter::FusedIterator, marker::PhantomData, ops::Deref};
use rayon::prelude::{IndexedParallelIterator, ParallelIterator};
use utils::bitset::BitSet;

use super::{Always, QueryFilter, Wrap, len};
use crate::{
    archetype::Archetype,
    layout::{LayoutAccess, QueryLayoutMut},
    mask::Mask,
    scene::Scene,
};

/// This is a query that will be fetched from the main scene that we can use to get components out of entries with a specific layout.
/// Even though I define the 'it, 'b, and 's lfietimes, I don't use them in this query, I only use them in the query iterator.
pub struct QueryMut<'s, L: QueryLayoutMut<'s>> {
    pub(super) archetypes: Vec<&'s mut Archetype>,
    pub(super) access: LayoutAccess,
    _phantom: PhantomData<&'s L>,
}

impl<'s, L: QueryLayoutMut<'s>> QueryMut<'s, L> {
    // Create a new mut query from the scene
    pub(crate) fn new(scene: &'s mut Scene) -> Self {
        let (access, archetypes, _) = super::archetypes_mut::<L, Always>(scene.archetypes_mut());

        Self {
            archetypes,
            access,
            _phantom: PhantomData,
        }
    }

    /*
    // Create a new mut query from the scene, but make it have a specific entry enable/disable masks
    pub(crate) fn new_with_filter<F: QueryFilter>(
        scene: &'s mut Scene,
        _: Wrap<F>,
        ticked: bool,
    ) -> Self {
        // Filter out the archetypes then create the bitsets
        let (access, archetypes, cached) = super::archetypes_mut::<L, F>(scene.archetypes_mut());
        let bitsets =
            super::generate_bitset_chunks::<F>(archetypes.iter().map(|a| &**a), cached, ticked);

        Self {
            archetypes,
            access,
            bitsets: Some(bitsets),
            _phantom1: PhantomData,
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