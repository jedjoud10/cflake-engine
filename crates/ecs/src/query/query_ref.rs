use utils::BitSet;

use crate::{Always, Archetype, LayoutAccess, QueryFilter, QueryLayoutRef, Scene, Wrap};
use std::{iter::FusedIterator, marker::PhantomData};

/// This is a query that will be fetched from the main scene that we can use to get components out of entries with a specific layout.
/// Even though I define the 'it, 'b, and 's lifetime, I don't use them in this query, I only use them in the query iterator.
pub struct QueryRef<'a: 'b, 'b, 's, L: QueryLayoutRef> {
    archetypes: Vec<&'a Archetype>,
    access: LayoutAccess,
    bitsets: Option<Vec<BitSet<usize>>>,
    _phantom1: PhantomData<&'b ()>,
    _phantom2: PhantomData<&'s ()>,
    _phantom3: PhantomData<L>,
}

impl<'a: 'b, 'b, 's, L: QueryLayoutRef> QueryRef<'a, 'b, 's, L> {
    // Create a new mut query from the scene for active entities
    pub(crate) fn new(scene: &'a Scene) -> Self {
        let (mask, archetypes, _) = super::archetypes::<L, Always>(scene.archetypes());
        Self {
            archetypes,
            bitsets: None,
            _phantom3: PhantomData,
            access: mask,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }

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
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }

    /// Get the access masks that we have calculated.
    pub fn layout_access(&self) -> LayoutAccess {
        self.access
    }

    /// Get the number of entries that we will have to iterate through.
    pub fn len(&self) -> usize {
        len(&self.archetypes, &self.bitsets)
    }

    /// Check if the query is empty.
    pub fn is_empty(&self) -> bool {
        self.archetypes.is_empty()
    }
}

// Calculate the number of elements there are in the archetypes, but also take in consideration
// the bitsets (if specified)
fn len(archetypes: &[&Archetype], bitsets: &Option<Vec<BitSet<usize>>>) -> usize {
    if let Some(bitsets) = bitsets {
        bitsets
            .iter()
            .zip(archetypes.iter())
            .map(|(b, a)| b.count_ones().min(a.len()))
            .sum()
    } else {
        archetypes.iter().map(|a| a.len()).sum()
    }
}

impl<'a: 'b, 'b, 'it, L: QueryLayoutRef> IntoIterator for QueryRef<'a, 'b, 'it, L> {
    type Item = L;
    type IntoIter = QueryRefIter<'b, L>;

    fn into_iter(self) -> Self::IntoIter {
        QueryRefIter {
            archetypes: self.archetypes,
            bitsets: self.bitsets,
            chunk: None,
            index: 0,
            _phantom2: PhantomData,
        }
    }
}

// Currently loaded chunk in the immutable query iterator
struct Chunk<L: QueryLayoutRef> {
    ptrs: L::PtrTuple,
    bitset: Option<BitSet<usize>>,
    length: usize,
}

/// This is a immutable query iterator that will iterate through all the query entries in arbitrary order.
pub struct QueryRefIter<'b, L: QueryLayoutRef> {
    // Inputs from the query
    archetypes: Vec<&'b Archetype>,
    bitsets: Option<Vec<BitSet<usize>>>,

    // Unique to the iterator
    chunk: Option<Chunk<L>>,
    index: usize,
    _phantom2: PhantomData<L>,
}

impl<'b, 's, L: QueryLayoutRef> QueryRefIter<'b, L> {
    // Hop onto the next archetype if we are done iterating through the current one
    fn check_hop_chunk(&mut self) -> Option<()> {
        let len = self
            .chunk
            .as_ref()
            .map(|chunk| chunk.length)
            .unwrap_or_default();

        if self.index + 1 > len {
            let archetype = self.archetypes.pop()?;
            let bitset = self.bitsets.as_mut().map(|vec| vec.pop().unwrap());
            let ptrs = unsafe { L::ptrs_from_archetype_unchecked(archetype) };
            let length = archetype.len();
            self.index = 0;
            self.chunk = Some(Chunk {
                ptrs,
                bitset,
                length,
            });
        }

        Some(())
    }
}

impl<'b, L: QueryLayoutRef> Iterator for QueryRefIter<'b, L> {
    type Item = L;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = len(&self.archetypes, &self.bitsets);
        (len, Some(len))
    }

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Always hop to the next chunk at the start of the hop iteration / normal iteration
            self.check_hop_chunk()?;

            if let Some(chunk) = &self.chunk {
                // Check for bitset
                if let Some(bitset) = &chunk.bitset {
                    // Check the next entry that is valid (that passed the filter)
                    if let Some(hop) = bitset.find_one_from(self.index) {
                        self.index = hop;
                        break;
                    } else {
                        // Hop to the next archetype if we could not find one
                        // This will force the iterator to hop to the next archetype
                        self.index = chunk.length;
                        continue;
                    }
                } else {
                    // If we do not have a bitset, don't do anything
                    break;
                }
            }
        }

        // I have to do this since iterators cannot return data that they are referencing, but in this case, it is safe to do so
        self.chunk.as_mut()?;
        let ptrs = self.chunk.as_ref().unwrap().ptrs;
        let items = unsafe { L::read_unchecked(ptrs, self.index) };
        self.index += 1;

        Some(items)
    }
}

impl<'b, L: QueryLayoutRef> ExactSizeIterator for QueryRefIter<'b, L> {}
impl<'b, 's, L: QueryLayoutRef> FusedIterator for QueryRefIter<'b, L> {}
