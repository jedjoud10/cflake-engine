use std::{marker::PhantomData, ops::Deref};
use utils::bitset::BitSet;

use crate::{
    archetype::{Archetype, StateColumn, UntypedColumn},
    layout::{LayoutAccess, QueryLayoutMut, QueryLayoutRef},
    mask::Mask,
    scene::ArchetypeSet,
};

use super::QueryFilter;


// Update the mutability state column of a specific archetype based on a masks' compound unit masks
pub(crate) fn apply_mutability_states(
    archetype: &mut Archetype,
    mutability: Mask,
    bitset: Option<&BitSet<u64>>,
) {
    let table = archetype.table_mut();
    for unit in mutability.units() {
        let column = table.get_mut(&unit).unwrap();
        let states = column.states_mut();

        if let Some(bitset) = bitset {
            for (out_states, in_states) in
                states.chunks_mut().iter_mut().zip(bitset.chunks().iter())
            {
                out_states.modified = *in_states;
            }
        } else {
            for out in states.chunks_mut() {
                out.modified = u64::MAX;
            }
        }
    }
}


// Given a scene and a specific filter, filter out the archetypes
// This will also prepare the filter for later by caching required data
// Only used internally by the mutable query
pub(super) fn archetypes_mut<'a, L: QueryLayoutMut<'a>, F: QueryFilter>(
    archetypes: &'a mut ArchetypeSet,
) -> (LayoutAccess, Vec<&mut Archetype>, F::Cached) {
    let mask = L::reduce(|a, b| a | b);
    let cached = F::prepare();
    let archetypes = archetypes
        .iter_mut()
        .filter_map(move |(&archetype_mask, archetype)| {
            (!archetype.is_empty() && archetype_mask.contains(mask.search())).then_some(archetype)
        })
        .filter(|a| F::evaluate_archetype(cached, a))
        .collect::<Vec<_>>();

    (mask, archetypes, cached)
}

// Given a scene and a specific filter, filter out the archetypes
// This will also prepare the filter for later by caching required data
// Only used internally by the immutable query
pub(super) fn archetypes<'a, L: QueryLayoutRef<'a>, F: QueryFilter>(
    archetypes: &'a ArchetypeSet,
) -> (LayoutAccess, Vec<&Archetype>, F::Cached) {
    let mask = L::reduce(|a, b| a | b);
    let cached = F::prepare();
    let archetypes = archetypes
        .iter()
        .filter_map(move |(&archetype_mask, archetype)| {
            (!archetype.is_empty() && archetype_mask.contains(mask.search())).then_some(archetype)
        })
        .filter(|a| F::evaluate_archetype(cached, a))
        .collect::<Vec<_>>();

    (mask, archetypes, cached)
}

// Sum the number of entities there are in the archetypes in total
pub(super) fn len<'a, A: Deref<Target = Archetype>>(archetypes: &[A]) -> usize {
    archetypes.iter().map(|a| a.deref().len()).sum()
}

// Create a vector of bitsets in case we are using query filtering
pub(super) fn generate_bitset_chunks<'a, F: QueryFilter>(
    archetypes: impl Iterator<Item = &'a Archetype>,
    cached: F::Cached,
) -> Vec<BitSet<u64>> {
    // Filter the entries by chunks of 64 entries at a time
    let iterator = archetypes.map(|archetype| {
        let columns = F::cache_columns(cached, archetype);
        let chunks = archetype.entities().len() as f32 / usize::BITS as f32;
        let chunks = chunks.ceil() as usize;
        BitSet::<u64>::from_chunks_iter(
            (0..chunks)
                .into_iter()
                .map(move |i| F::evaluate_chunk(&columns, i)),
        )
    });

    // Create a unique hop bitset for each archetype
    Vec::from_iter(iterator)
}
