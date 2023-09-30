use std::{marker::PhantomData, ops::Deref};
use utils::bitset::BitSet;

use crate::{
    archetype::{Archetype, StateColumn, UntypedColumn},
    layout::{LayoutAccess, QueryLayoutMut, QueryLayoutRef},
    mask::Mask,
    scene::ArchetypeSet,
};

/// Result value whenever we call evaluate_chunk within QueryFilter.
pub struct ChunkEval(u64);

/// Basic evaluator that will be implemented for the filter sources and modifiers.
/// These filters allow users to discard certain entries when iterating.
pub trait QueryFilter {
    /// Cached data for fast traversal (stores the bitmask of a specific component for eaxmple).
    type Cached: 'static + Clone + Copy;

    /// Cached columns that we fetch from an archetypes.
    type Columns<'a>: 'a;

    /// Create the permanent cached data.
    fn prepare() -> Self::Cached;

    /// Evaluate a single archetype to check if it passes the filter.
    fn evaluate_archetype(cached: Self::Cached, archetype: &Archetype) -> bool;

    /// Cache the state columns of a specific archetype
    fn cache_columns<'a>(
        cached: Self::Cached,
        archetype: &'a Archetype,
        ticked: bool,
    ) -> Self::Columns<'a>;

    /// Evaluate a single chunk to check if all the entries within it pass the filter.
    /// When the bit is set, it means that the entry passed. If it's not set, then the entry didn't pass the filter.
    fn evaluate_chunk(columns: &Self::Columns<'_>, index: usize) -> u64;
}

/// We need a wrapper to be able to implemented the rust bitwise operators.
pub struct Wrap<T: QueryFilter>(PhantomData<T>);

impl<T: QueryFilter> Clone for Wrap<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: QueryFilter> Copy for Wrap<T> {}

/// Filter sources that passes if the [QueryLayoutRef] was added into the entities
/// All the components within the [QueryLayoutRef] must be within the archetype for this filter to pass the coarse test
pub struct Added<T: QueryLayoutRef>(PhantomData<T>);

/// Filter sources that passes if the [QueryLayoutRef] was modified before the query was executed
/// All the components within the [QueryLayoutRef] must be within the archetype for this filter to pass the coarse test
pub struct Modified<T: QueryLayoutRef>(PhantomData<T>);

/// Filter sources that passes if the [QueryLayoutRef] is used by the entities
/// All the components within the [QueryLayoutRef] must be within the archetype for this filter to pass the coarse test
pub struct Contains<T: QueryLayoutRef>(PhantomData<T>);

// Note: ONLY USED INTERNALLY. THIS IS LITERALLY USELESS
pub(crate) struct Always;

/// Passes if both filters pass the coarse / fine tests
pub struct And<A: QueryFilter, B: QueryFilter>(PhantomData<A>, PhantomData<B>);

/// Passes if any of the filters pass the coarse / fine tests
pub struct Or<A: QueryFilter, B: QueryFilter>(PhantomData<A>, PhantomData<B>);

/// Passes if only one of the filters pass the coarse / fine tests
pub struct Xor<A: QueryFilter, B: QueryFilter>(PhantomData<A>, PhantomData<B>);

/// Passes if the filters fail the coarse / fine tests
pub struct Not<A: QueryFilter>(PhantomData<A>);

pub(crate) fn get_either_states(col: &UntypedColumn, ticked: bool) -> &StateColumn {
    match ticked {
        true => col.delta_tick_states(),
        false => col.delta_frame_states(),
    }
}

pub(crate) fn get_either_states_mut(col: &mut UntypedColumn, ticked: bool) -> &mut StateColumn {
    match ticked {
        true => col.delta_tick_states_mut(),
        false => col.delta_frame_states_mut(),
    }
}

impl<L: QueryLayoutRef> QueryFilter for Added<L> {
    type Cached = Mask;
    type Columns<'a> = Vec<Option<&'a StateColumn>>;

    fn prepare() -> Self::Cached {
        LayoutAccess::from_layout_ref::<L>().search()
    }

    fn evaluate_archetype(cached: Self::Cached, archetype: &Archetype) -> bool {
        archetype.mask().contains(cached)
    }

    fn cache_columns(
        cached: Self::Cached,
        archetype: &Archetype,
        ticked: bool,
    ) -> Self::Columns<'_> {
        cached
            .units()
            .map(|unit| {
                archetype
                    .table()
                    .get(&unit)
                    .map(|col| get_either_states(col, ticked))
            })
            .collect::<Vec<_>>()
    }

    fn evaluate_chunk(columns: &Self::Columns<'_>, index: usize) -> u64 {
        // IDK IF THIS WORKS IT MAKES SENSE ON PAPER THO
        // FIXME: MUST TEST
        columns
            .iter()
            .map(|c| {
                c.map(|c| c.get_chunk(index).unwrap().added)
                    .unwrap_or_default()
            })
            .reduce(|a, b| a & b)
            .unwrap_or_default()
    }
}

impl<L: QueryLayoutRef> QueryFilter for Modified<L> {
    type Cached = Mask;
    type Columns<'a> = Vec<Option<&'a StateColumn>>;

    fn prepare() -> Self::Cached {
        LayoutAccess::from_layout_ref::<L>().search()
    }

    fn evaluate_archetype(cached: Self::Cached, archetype: &Archetype) -> bool {
        archetype.mask().contains(cached)
    }

    fn cache_columns(
        cached: Self::Cached,
        archetype: &Archetype,
        ticked: bool,
    ) -> Self::Columns<'_> {
        cached
            .units()
            .map(|unit| {
                archetype
                    .table()
                    .get(&unit)
                    .map(|col| get_either_states(col, ticked))
            })
            .collect::<Vec<_>>()
    }

    fn evaluate_chunk(columns: &Self::Columns<'_>, index: usize) -> u64 {
        // IDK IF THIS WORKS IT MAKES SENSE ON PAPER THO
        // FIXME: MUST TEST
        columns
            .iter()
            .map(|c| {
                c.map(|c| c.get_chunk(index).unwrap().modified)
                    .unwrap_or_default()
            })
            .reduce(|a, b| a & b)
            .unwrap_or_default()
        
    }
}

impl<L: QueryLayoutRef> QueryFilter for Contains<L> {
    type Cached = Mask;
    type Columns<'a> = ();

    fn prepare() -> Self::Cached {
        LayoutAccess::from_layout_ref::<L>().search()
    }

    fn evaluate_archetype(cached: Self::Cached, archetype: &Archetype) -> bool {
        archetype.mask().contains(cached)
    }

    fn cache_columns(
        _cached: Self::Cached,
        _archetype: &Archetype,
        _ticked: bool,
    ) -> Self::Columns<'_> {
    }

    fn evaluate_chunk(_columns: &Self::Columns<'_>, _index: usize) -> u64 {
        u64::MAX
    }
}

impl QueryFilter for Always {
    type Cached = ();
    type Columns<'a> = ();

    fn prepare() -> Self::Cached {}

    fn evaluate_archetype(_cached: Self::Cached, _archetype: &Archetype) -> bool {
        true
    }

    fn cache_columns(
        _cached: Self::Cached,
        _archetype: &Archetype,
        _ticked: bool,
    ) -> Self::Columns<'_> {
    }

    fn evaluate_chunk(_columns: &Self::Columns<'_>, _index: usize) -> u64 {
        u64::MAX
    }
}

// Trait implementations for modifiers
impl<A: QueryFilter, B: QueryFilter> QueryFilter for And<A, B> {
    type Cached = (A::Cached, B::Cached);
    type Columns<'a> = (A::Columns<'a>, B::Columns<'a>);

    fn prepare() -> Self::Cached {
        (A::prepare(), B::prepare())
    }

    fn evaluate_archetype(cached: Self::Cached, archetype: &Archetype) -> bool {
        A::evaluate_archetype(cached.0, archetype) && B::evaluate_archetype(cached.1, archetype)
    }

    fn cache_columns(
        cached: Self::Cached,
        archetype: &Archetype,
        ticked: bool,
    ) -> Self::Columns<'_> {
        (
            A::cache_columns(cached.0, archetype, ticked),
            B::cache_columns(cached.1, archetype, ticked),
        )
    }

    fn evaluate_chunk(columns: &Self::Columns<'_>, index: usize) -> u64 {
        let a = A::evaluate_chunk(&columns.0, index);
        let b = B::evaluate_chunk(&columns.1, index);
        a & b
    }
}

impl<A: QueryFilter, B: QueryFilter> QueryFilter for Or<A, B> {
    type Cached = (A::Cached, B::Cached);
    type Columns<'a> = (A::Columns<'a>, B::Columns<'a>);

    fn prepare() -> Self::Cached {
        (A::prepare(), B::prepare())
    }

    fn evaluate_archetype(cached: Self::Cached, archetype: &Archetype) -> bool {
        A::evaluate_archetype(cached.0, archetype) || B::evaluate_archetype(cached.1, archetype)
    }

    fn cache_columns(
        cached: Self::Cached,
        archetype: &Archetype,
        ticked: bool,
    ) -> Self::Columns<'_> {
        (
            A::cache_columns(cached.0, archetype, ticked),
            B::cache_columns(cached.1, archetype, ticked),
        )
    }

    fn evaluate_chunk(columns: &Self::Columns<'_>, index: usize) -> u64 {
        let a = A::evaluate_chunk(&columns.0, index);
        let b = B::evaluate_chunk(&columns.1, index);
        a | b
    }
}

impl<A: QueryFilter, B: QueryFilter> QueryFilter for Xor<A, B> {
    type Cached = (A::Cached, B::Cached);
    type Columns<'a> = (A::Columns<'a>, B::Columns<'a>);

    fn prepare() -> Self::Cached {
        (A::prepare(), B::prepare())
    }

    fn evaluate_archetype(cached: Self::Cached, archetype: &Archetype) -> bool {
        A::evaluate_archetype(cached.0, archetype) ^ B::evaluate_archetype(cached.1, archetype)
    }

    fn cache_columns(
        cached: Self::Cached,
        archetype: &Archetype,
        ticked: bool,
    ) -> Self::Columns<'_> {
        (
            A::cache_columns(cached.0, archetype, ticked),
            B::cache_columns(cached.1, archetype, ticked),
        )
    }

    fn evaluate_chunk(columns: &Self::Columns<'_>, index: usize) -> u64 {
        let a = A::evaluate_chunk(&columns.0, index);
        let b = B::evaluate_chunk(&columns.1, index);
        a ^ b
    }
}

impl<A: QueryFilter> QueryFilter for Not<A> {
    type Cached = A::Cached;
    type Columns<'a> = A::Columns<'a>;

    fn prepare() -> Self::Cached {
        A::prepare()
    }

    fn evaluate_archetype(cached: Self::Cached, archetype: &Archetype) -> bool {
        !A::evaluate_archetype(cached, archetype)
    }

    fn cache_columns(
        cached: Self::Cached,
        archetype: &Archetype,
        ticked: bool,
    ) -> Self::Columns<'_> {
        A::cache_columns(cached, archetype, ticked)
    }

    fn evaluate_chunk(columns: &Self::Columns<'_>, index: usize) -> u64 {
        !A::evaluate_chunk(columns, index)
    }
}

/// Source to check if we have modified a specific component before this call.
pub fn modified<L: QueryLayoutRef>() -> Wrap<Modified<L>> {
    Wrap::<Modified<L>>(PhantomData)
}

/// Source to check if we added a specific component before this call.
pub fn added<L: QueryLayoutRef>() -> Wrap<Added<L>> {
    Wrap::<Added<L>>(PhantomData)
}

/// Source to check if we contain a specific component within the archetype.
pub fn contains<L: QueryLayoutRef>() -> Wrap<Contains<L>> {
    Wrap::<Contains<L>>(PhantomData)
}

impl<A: QueryFilter, B: QueryFilter> std::ops::BitAnd<Wrap<B>> for Wrap<A> {
    type Output = Wrap<And<A, B>>;

    fn bitand(self, _: Wrap<B>) -> Self::Output {
        Wrap(PhantomData)
    }
}

impl<A: QueryFilter, B: QueryFilter> std::ops::BitOr<Wrap<B>> for Wrap<A> {
    type Output = Wrap<Or<A, B>>;

    fn bitor(self, _: Wrap<B>) -> Self::Output {
        Wrap(PhantomData)
    }
}

impl<A: QueryFilter, B: QueryFilter> std::ops::BitXor<Wrap<B>> for Wrap<A> {
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
