use crate::{
    registry::{self},
    Archetype, ArchetypeSet, Component, LayoutAccess, Mask,
    QueryLayoutMut, QueryLayoutRef, StateColumn,
};
use std::marker::PhantomData;
use utils::BitSet;

// Result value whenever we call evaluate_chunk within QueryFilter
// This is an enum because in the Contains<T> filter source we want to not care about the chunk evaluation
pub enum ChunkEval {
    // Chunk evaluation that gave us a predictable answer that is taken acount when using modifiers
    Evaluated(usize),

    // Value that isn't needed or that SHOULD NOT be taken account when using modifiers
    // This will propagate up whenever we use combinational modifiers, and it will always return true
    Passthrough,
}

impl ChunkEval {
    // Same function as Option::zip_with, but stable
    pub fn zip_with<F: FnOnce((usize, usize)) -> usize>(
        self,
        other: Self,
        fun: F,
    ) -> Self {
        match (self, other) {
            (ChunkEval::Evaluated(a), ChunkEval::Evaluated(b)) => {
                Self::Evaluated(fun((a, b)))
            }
            _ => Self::Passthrough,
        }
    }

    // Map, only used for modifiers
    pub fn map<F: FnOnce(usize) -> usize>(self, fun: F) -> Self {
        match self {
            ChunkEval::Evaluated(x) => Self::Evaluated(fun(x)),
            ChunkEval::Passthrough => Self::Passthrough,
        }
    }

    // Return the inner value if Valid, or return usize::MAX if DontCare
    pub fn into_inner(self) -> usize {
        match self {
            ChunkEval::Evaluated(x) => x,
            ChunkEval::Passthrough => usize::MAX,
        }
    }
}

// Basic evaluator that will be implemented for the filter sources and modifiers
// These filters allow users to discard certain entries when iterating
pub trait QueryFilter: 'static {
    // Cached data for fast traversal (only stores the bitmask of a specific component)
    type Cached: 'static + Clone + Copy;

    // Cached columns that we fetch from an archetypes
    type Columns<'a>: 'a + Clone + Copy;

    // Create the permanent cached data
    fn prepare() -> Self::Cached;

    // Evaluate a single archetype to check if it passes the filter
    fn evaluate_archetype(
        cached: Self::Cached,
        archetype: &Archetype,
    ) -> bool;

    // Cache the state columns of a specific archetype
    fn cache_columns<'a>(
        cached: Self::Cached,
        archetype: &'a Archetype,
    ) -> Self::Columns<'a>;

    // Evaluate a single chunk to check if all the entries within it pass the filter
    // When the bit is set, it means that the entry passed. If it's not set, then the entry didn't pass the filter
    fn evaluate_chunk(
        columns: Self::Columns<'_>,
        index: usize,
    ) -> ChunkEval;
}

// Given a scene and a specific filter, filter out the archetypes
// This will also prepare the filter for later by caching required data
// Only used internally by the mutable query
pub(super) fn archetypes_mut<L: QueryLayoutMut, F: QueryFilter>(
    archetypes: &mut ArchetypeSet,
) -> (LayoutAccess, Vec<&mut Archetype>, F::Cached) {
    let mask = L::reduce(|a, b| a | b);
    let cached = F::prepare();
    let archetypes = archetypes
        .iter_mut()
        .filter_map(move |(&archetype_mask, archetype)| {
            (!archetype.is_empty()
                && archetype_mask.contains(mask.search()))
            .then_some(archetype)
        })
        .filter(|a| F::evaluate_archetype(cached, a))
        .collect::<Vec<_>>();

    (mask, archetypes, cached)
}

// Given a scene and a specific filter, filter out the archetypes
// This will also prepare the filter for later by caching required data
// Only used internally by the immutable query
pub(super) fn archetypes<L: QueryLayoutRef, F: QueryFilter>(
    archetypes: &ArchetypeSet,
) -> (LayoutAccess, Vec<&Archetype>, F::Cached) {
    let mask = L::reduce(|a, b| a | b);
    let cached = F::prepare();
    let archetypes = archetypes
        .iter()
        .filter_map(move |(&archetype_mask, archetype)| {
            (!archetype.is_empty()
                && archetype_mask.contains(mask.search()))
            .then_some(archetype)
        })
        .filter(|a| F::evaluate_archetype(cached, a))
        .collect::<Vec<_>>();

    (mask, archetypes, cached)
}

// Create a vector of bitsets in case we are using query filtering
pub(super) fn generate_bitset_chunks<'a, F: QueryFilter>(
    archetypes: impl Iterator<Item = &'a Archetype>,
    cached: F::Cached,
) -> Vec<BitSet> {
    // Filter the entries by chunks of 64 entries at a time
    let iterator =
        archetypes.map(|archetype| {
            let columns = F::cache_columns(cached, archetype);
            let chunks = archetype.entities().len() as f32
                / usize::BITS as f32;
            let chunks = chunks.ceil() as usize;
            BitSet::from_chunks_iter((0..chunks).into_iter().map(
                move |i| F::evaluate_chunk(columns, i).into_inner(),
            ))
        });

    // Create a unique hop bitset for each archetype
    Vec::from_iter(iterator)
}

// We need a wrapper to be able to implemented the rust bitwise operators
pub struct Wrap<T: QueryFilter>(PhantomData<T>);

// Filter sources based on components
pub struct Added<T: Component>(PhantomData<T>);
pub struct Modified<T: Component>(PhantomData<T>);
pub struct Contains<T: Component>(PhantomData<T>);

// Note: ONLY USED INTERNALLY. THIS IS LITERALLY USELESS
pub(crate) struct Always(());

// Query filter operators
pub struct And<A: QueryFilter, B: QueryFilter>(
    PhantomData<A>,
    PhantomData<B>,
);
pub struct Or<A: QueryFilter, B: QueryFilter>(
    PhantomData<A>,
    PhantomData<B>,
);
pub struct Xor<A: QueryFilter, B: QueryFilter>(
    PhantomData<A>,
    PhantomData<B>,
);
pub struct Not<A: QueryFilter>(PhantomData<A>);

impl<T: Component> QueryFilter for Added<T> {
    type Cached = Mask;
    type Columns<'a> = Option<&'a StateColumn>;

    fn prepare() -> Self::Cached {
        registry::mask::<T>()
    }

    fn evaluate_archetype(
        cached: Self::Cached,
        archetype: &Archetype,
    ) -> bool {
        archetype.mask().contains(cached)
    }

    fn cache_columns(
        cached: Self::Cached,
        archetype: &Archetype,
    ) -> Self::Columns<'_> {
        archetype.table().get(&cached).map(|col| col.states())
    }

    fn evaluate_chunk(
        columns: Self::Columns<'_>,
        index: usize,
    ) -> ChunkEval {
        ChunkEval::Evaluated(
            columns
                .map(|c| c.get_chunk(index).unwrap().added)
                .unwrap_or_default(),
        )
    }
}

impl<T: Component> QueryFilter for Modified<T> {
    type Cached = Mask;
    type Columns<'a> = Option<&'a StateColumn>;

    fn prepare() -> Self::Cached {
        registry::mask::<T>()
    }

    fn evaluate_archetype(
        cached: Self::Cached,
        archetype: &Archetype,
    ) -> bool {
        archetype.mask().contains(cached)
    }

    fn cache_columns(
        cached: Self::Cached,
        archetype: &Archetype,
    ) -> Self::Columns<'_> {
        archetype.table().get(&cached).map(|col| col.states())
    }

    fn evaluate_chunk(
        columns: Self::Columns<'_>,
        index: usize,
    ) -> ChunkEval {
        ChunkEval::Evaluated(
            columns
                .map(|c| c.get_chunk(index).unwrap().modified)
                .unwrap_or_default(),
        )
    }
}

impl<T: Component> QueryFilter for Contains<T> {
    type Cached = Mask;
    type Columns<'a> = ();

    fn prepare() -> Self::Cached {
        registry::mask::<T>()
    }

    fn evaluate_archetype(
        cached: Self::Cached,
        archetype: &Archetype,
    ) -> bool {
        archetype.mask().contains(cached)
    }

    fn cache_columns(
        _cached: Self::Cached,
        _archetype: &Archetype,
    ) -> Self::Columns<'_> {
    }

    fn evaluate_chunk(
        _columns: Self::Columns<'_>,
        _index: usize,
    ) -> ChunkEval {
        ChunkEval::Passthrough
    }
}

impl QueryFilter for Always {
    type Cached = ();
    type Columns<'a> = ();

    fn prepare() -> Self::Cached {}

    fn evaluate_archetype(
        _cached: Self::Cached,
        _archetype: &Archetype,
    ) -> bool {
        true
    }

    fn cache_columns(
        _cached: Self::Cached,
        _archetype: &Archetype,
    ) -> Self::Columns<'_> {
    }

    fn evaluate_chunk(
        _columns: Self::Columns<'_>,
        _index: usize,
    ) -> ChunkEval {
        ChunkEval::Evaluated(usize::MAX)
    }
}

// Trait implementations for modifiers
impl<A: QueryFilter, B: QueryFilter> QueryFilter for And<A, B> {
    type Cached = (A::Cached, B::Cached);
    type Columns<'a> = (A::Columns<'a>, B::Columns<'a>);

    fn prepare() -> Self::Cached {
        (A::prepare(), B::prepare())
    }

    fn evaluate_archetype(
        cached: Self::Cached,
        archetype: &Archetype,
    ) -> bool {
        A::evaluate_archetype(cached.0, archetype)
            && B::evaluate_archetype(cached.1, archetype)
    }

    fn cache_columns(
        cached: Self::Cached,
        archetype: &Archetype,
    ) -> Self::Columns<'_> {
        (
            A::cache_columns(cached.0, archetype),
            B::cache_columns(cached.1, archetype),
        )
    }

    fn evaluate_chunk(
        columns: Self::Columns<'_>,
        index: usize,
    ) -> ChunkEval {
        let a = A::evaluate_chunk(columns.0, index);
        let b = B::evaluate_chunk(columns.1, index);
        a.zip_with(b, |(a, b)| a & b)
    }
}

impl<A: QueryFilter, B: QueryFilter> QueryFilter for Or<A, B> {
    type Cached = (A::Cached, B::Cached);
    type Columns<'a> = (A::Columns<'a>, B::Columns<'a>);

    fn prepare() -> Self::Cached {
        (A::prepare(), B::prepare())
    }

    fn evaluate_archetype(
        cached: Self::Cached,
        archetype: &Archetype,
    ) -> bool {
        A::evaluate_archetype(cached.0, archetype)
            || B::evaluate_archetype(cached.1, archetype)
    }

    fn cache_columns(
        cached: Self::Cached,
        archetype: &Archetype,
    ) -> Self::Columns<'_> {
        (
            A::cache_columns(cached.0, archetype),
            B::cache_columns(cached.1, archetype),
        )
    }

    fn evaluate_chunk(
        columns: Self::Columns<'_>,
        index: usize,
    ) -> ChunkEval {
        let a = A::evaluate_chunk(columns.0, index);
        let b = B::evaluate_chunk(columns.1, index);
        a.zip_with(b, |(a, b)| a | b)
    }
}

impl<A: QueryFilter, B: QueryFilter> QueryFilter for Xor<A, B> {
    type Cached = (A::Cached, B::Cached);
    type Columns<'a> = (A::Columns<'a>, B::Columns<'a>);

    fn prepare() -> Self::Cached {
        (A::prepare(), B::prepare())
    }

    fn evaluate_archetype(
        cached: Self::Cached,
        archetype: &Archetype,
    ) -> bool {
        A::evaluate_archetype(cached.0, archetype)
            ^ B::evaluate_archetype(cached.1, archetype)
    }

    fn cache_columns(
        cached: Self::Cached,
        archetype: &Archetype,
    ) -> Self::Columns<'_> {
        (
            A::cache_columns(cached.0, archetype),
            B::cache_columns(cached.1, archetype),
        )
    }

    fn evaluate_chunk(
        columns: Self::Columns<'_>,
        index: usize,
    ) -> ChunkEval {
        let a = A::evaluate_chunk(columns.0, index);
        let b = B::evaluate_chunk(columns.1, index);
        a.zip_with(b, |(a, b)| a ^ b)
    }
}

impl<A: QueryFilter> QueryFilter for Not<A> {
    type Cached = A::Cached;
    type Columns<'a> = A::Columns<'a>;

    fn prepare() -> Self::Cached {
        A::prepare()
    }

    fn evaluate_archetype(
        cached: Self::Cached,
        archetype: &Archetype,
    ) -> bool {
        !A::evaluate_archetype(cached, archetype)
    }

    fn cache_columns(
        cached: Self::Cached,
        archetype: &Archetype,
    ) -> Self::Columns<'_> {
        A::cache_columns(cached, archetype)
    }

    fn evaluate_chunk(
        columns: Self::Columns<'_>,
        index: usize,
    ) -> ChunkEval {
        A::evaluate_chunk(columns, index).map(|x| !x)
    }
}

// Source to check if we have modified a specific component before this call
pub fn modified<T: Component>() -> Wrap<Modified<T>> {
    Wrap::<Modified<T>>(PhantomData)
}

// Source to check if we added a specific component before this call
pub fn added<T: Component>() -> Wrap<Added<T>> {
    Wrap::<Added<T>>(PhantomData)
}

// Source to check if we contain a specific component within the archetype
pub fn contains<T: Component>() -> Wrap<Contains<T>> {
    Wrap::<Contains<T>>(PhantomData)
}

impl<A: QueryFilter, B: QueryFilter> std::ops::BitAnd<Wrap<B>>
    for Wrap<A>
{
    type Output = Wrap<And<A, B>>;

    fn bitand(self, _: Wrap<B>) -> Self::Output {
        Wrap(PhantomData)
    }
}

impl<A: QueryFilter, B: QueryFilter> std::ops::BitOr<Wrap<B>>
    for Wrap<A>
{
    type Output = Wrap<Or<A, B>>;

    fn bitor(self, _: Wrap<B>) -> Self::Output {
        Wrap(PhantomData)
    }
}

impl<A: QueryFilter, B: QueryFilter> std::ops::BitXor<Wrap<B>>
    for Wrap<A>
{
    type Output = Wrap<Or<A, B>>;

    fn bitxor(self, _: Wrap<B>) -> Self::Output {
        Wrap(PhantomData)
    }
}

impl<A: QueryFilter> std::ops::Not for Wrap<A> {
    type Output = Wrap<Not<A>>;

    fn not(self) -> Self::Output {
        Wrap(PhantomData)
    }
}
