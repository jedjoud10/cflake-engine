use crate::{
    registry::{self},
    Archetype, Component, Mask, StateColumn,
};
use std::{marker::PhantomData, ops::BitAnd};

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
    fn evaluate_archetype(cached: Self::Cached, archetype: &Archetype) -> bool;

    // Cache the state columns of a specific archetype
    fn cache_columns<'a>(cached: Self::Cached, archetype: &'a Archetype) -> Self::Columns<'a>;
    
    // Evaluate a single chunk to check if all the entries within it pass the filter
    // When the bit is set, it means that the entry passed. If it's not set, then the entry didn't pass the filter
    fn evaluate_chunk(columns: Self::Columns<'_>, index: usize) -> usize;
}

// We need a wrapper to be able to implemented the rust bitwise operators
pub struct Wrap<T: QueryFilter>(PhantomData<T>);

// Filter sources based on components
pub struct Added<T: Component>(PhantomData<T>);
pub struct Removed<T: Component>(PhantomData<T>);
pub struct Modified<T: Component>(PhantomData<T>);
pub struct Contains<T: Component>(PhantomData<T>);

// Constant source that always fail / succeed the test
pub struct Always(());
pub struct Never(());

// Query filter operators
pub struct And<A: QueryFilter, B: QueryFilter>(PhantomData<A>, PhantomData<B>);
pub struct Or<A: QueryFilter, B: QueryFilter>(PhantomData<A>, PhantomData<B>);
pub struct Xor<A: QueryFilter, B: QueryFilter>(PhantomData<A>, PhantomData<B>);
pub struct Not<A: QueryFilter>(PhantomData<A>);

impl<T: Component> QueryFilter for Added<T> {
    type Cached = Mask;
    type Columns<'a> = &'a StateColumn;

    fn prepare() -> Self::Cached {
        registry::mask::<T>()
    }

    fn evaluate_archetype(cached: Self::Cached, archetype: &Archetype) -> bool {
        true
    }

    fn cache_columns<'a>(cached: Self::Cached, archetype: &'a Archetype) -> Self::Columns<'a> {
        archetype.state_table().get(&cached).unwrap()
    }

    fn evaluate_chunk(columns: Self::Columns<'_>, index: usize) -> usize {
        columns.get(index).unwrap().added
    }
}

impl<T: Component> QueryFilter for Removed<T> {
    type Cached = Mask;
    type Columns<'a> = &'a StateColumn;

    fn prepare() -> Self::Cached {
        registry::mask::<T>()
    }

    fn evaluate_archetype(cached: Self::Cached, archetype: &Archetype) -> bool {
        true
    }

    fn cache_columns<'a>(cached: Self::Cached, archetype: &'a Archetype) -> Self::Columns<'a> {
        archetype.state_table().get(&cached).unwrap()
    }

    fn evaluate_chunk(columns: Self::Columns<'_>, index: usize) -> usize {
        columns.get(index).unwrap().removed
    }
}

impl<T: Component> QueryFilter for Modified<T> {
    type Cached = Mask;
    type Columns<'a> = &'a StateColumn;

    fn prepare() -> Self::Cached {
        registry::mask::<T>()
    }

    fn evaluate_archetype(cached: Self::Cached, archetype: &Archetype) -> bool {
        true
    }

    fn cache_columns<'a>(cached: Self::Cached, archetype: &'a Archetype) -> Self::Columns<'a> {
        archetype.state_table().get(&cached).unwrap()
    }

    fn evaluate_chunk(columns: Self::Columns<'_>, index: usize) -> usize {
        columns.get(index).unwrap().modified
    }
}

impl<T: Component> QueryFilter for Contains<T> {
    type Cached = Mask;
    type Columns<'a> = &'a StateColumn;

    fn prepare() -> Self::Cached {
        registry::mask::<T>()
    }

    fn evaluate_archetype(cached: Self::Cached, archetype: &Archetype) -> bool {
        archetype.mask().contains(cached)
    }

    fn cache_columns<'a>(cached: Self::Cached, archetype: &'a Archetype) -> Self::Columns<'a> {
        panic!()
    }

    fn evaluate_chunk(columns: Self::Columns<'_>, index: usize) -> usize {
        panic!()
    }
}

impl QueryFilter for Always {
    type Cached = ();
    type Columns<'a> = ();

    fn prepare() -> Self::Cached {
        ()
    }

    fn evaluate_archetype(cached: Self::Cached, archetype: &Archetype) -> bool {
        true
    }

    fn cache_columns<'a>(cached: Self::Cached, archetype: &'a Archetype) -> Self::Columns<'a> {
        ()
    }

    fn evaluate_chunk(columns: Self::Columns<'_>, index: usize) -> usize {
        usize::MAX
    }
}

impl QueryFilter for Never {
    type Cached = ();
    type Columns<'a> = ();

    fn prepare() -> Self::Cached {
        ()
    }

    fn evaluate_archetype(cached: Self::Cached, archetype: &Archetype) -> bool {
        false
    }

    fn cache_columns<'a>(cached: Self::Cached, archetype: &'a Archetype) -> Self::Columns<'a> {
        panic!()
    }

    fn evaluate_chunk(columns: Self::Columns<'_>, index: usize) -> usize {
        panic!()
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

    fn cache_columns<'a>(cached: Self::Cached, archetype: &'a Archetype) -> Self::Columns<'a> {
        (A::cache_columns(cached.0, archetype), B::cache_columns(cached.1, archetype))
    }

    fn evaluate_chunk(columns: Self::Columns<'_>, index: usize) -> usize {
        A::evaluate_chunk(columns.0, index) & B::evaluate_chunk(columns.1, index)
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

    fn cache_columns<'a>(cached: Self::Cached, archetype: &'a Archetype) -> Self::Columns<'a> {
        (A::cache_columns(cached.0, archetype), B::cache_columns(cached.1, archetype))
    }

    fn evaluate_chunk(columns: Self::Columns<'_>, index: usize) -> usize {
        A::evaluate_chunk(columns.0, index) | B::evaluate_chunk(columns.1, index)
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

    fn cache_columns<'a>(cached: Self::Cached, archetype: &'a Archetype) -> Self::Columns<'a> {
        (A::cache_columns(cached.0, archetype), B::cache_columns(cached.1, archetype))
    }

    fn evaluate_chunk(columns: Self::Columns<'_>, index: usize) -> usize {
        A::evaluate_chunk(columns.0, index) ^ B::evaluate_chunk(columns.1, index)
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

    fn cache_columns<'a>(cached: Self::Cached, archetype: &'a Archetype) -> Self::Columns<'a> {
        A::cache_columns(cached, archetype)
    }

    fn evaluate_chunk(columns: Self::Columns<'_>, index: usize) -> usize {
        !A::evaluate_chunk(columns, index)
    }
}

// Functions to create the sources and modifiers
pub fn modified<T: Component>() -> Wrap<Modified<T>> {
    Wrap::<Modified<T>>(PhantomData)
}
pub fn removed<T: Component>() -> Wrap<Removed<T>> {
    Wrap::<Removed<T>>(PhantomData)
}
pub fn added<T: Component>() -> Wrap<Added<T>> {
    Wrap::<Added<T>>(PhantomData)
}
pub fn contains<T: Component>() -> Wrap<Contains<T>> {
    Wrap::<Contains<T>>(PhantomData)
}

// Constant sources
pub fn always() -> Wrap<Always> {
    Wrap::<Always>(PhantomData)
}
pub fn never() -> Wrap<Never> {
    Wrap::<Never>(PhantomData)
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