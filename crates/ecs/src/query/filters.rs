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

    // Cached chunks that we fetch from an archetypes
    type Chunks<'a>: 'a + Clone + Copy;

    // Create the permanent cached data
    fn prepare() -> Self::Cached;

    // Evaluate a single archetype to check if it passes the filter
    fn eval_archetype(cached: Self::Cached, archetype: &Archetype) -> bool;

    // Cache the state columns of a specific archetype
    fn cache_chunks<'a>(cached: Self::Cached, archetype: &'a Archetype) -> Self::Chunks<'a>;
    
    // Evaluate a single chunk to check if all the entries within it pass the filter
    fn eval_chunk(chunks: Self::Chunks<'_>, index: usize) -> u64;
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
    type Chunks<'a> = &'a StateColumn;

    fn prepare() -> Self::Cached {
        registry::mask::<T>()
    }

    fn eval_archetype(cached: Self::Cached, archetype: &Archetype) -> bool {
        true
    }

    fn cache_chunks<'a>(cached: Self::Cached, archetype: &'a Archetype) -> Self::Chunks<'a> {
        archetype.state_table().get(&cached).unwrap()
    }

    fn eval_chunk(chunks: Self::Chunks<'_>, index: usize) -> u64 {
        chunks.get(index).unwrap().added
    }
}

impl<T: Component> QueryFilter for Removed<T> {
    type Cached = Mask;
    type Chunks<'a> = &'a StateColumn;

    fn prepare() -> Self::Cached {
        registry::mask::<T>()
    }

    fn eval_archetype(cached: Self::Cached, archetype: &Archetype) -> bool {
        true
    }

    fn cache_chunks<'a>(cached: Self::Cached, archetype: &'a Archetype) -> Self::Chunks<'a> {
        archetype.state_table().get(&cached).unwrap()
    }

    fn eval_chunk(chunks: Self::Chunks<'_>, index: usize) -> u64 {
        chunks.get(index).unwrap().removed
    }
}

impl<T: Component> QueryFilter for Modified<T> {
    type Cached = Mask;
    type Chunks<'a> = &'a StateColumn;

    fn prepare() -> Self::Cached {
        registry::mask::<T>()
    }

    fn eval_archetype(cached: Self::Cached, archetype: &Archetype) -> bool {
        true
    }

    fn cache_chunks<'a>(cached: Self::Cached, archetype: &'a Archetype) -> Self::Chunks<'a> {
        archetype.state_table().get(&cached).unwrap()
    }

    fn eval_chunk(chunks: Self::Chunks<'_>, index: usize) -> u64 {
        chunks.get(index).unwrap().modified
    }
}

impl<T: Component> QueryFilter for Contains<T> {
    type Cached = Mask;
    type Chunks<'a> = &'a StateColumn;

    fn prepare() -> Self::Cached {
        registry::mask::<T>()
    }

    fn eval_archetype(cached: Self::Cached, archetype: &Archetype) -> bool {
        archetype.mask().contains(cached)
    }

    fn cache_chunks<'a>(cached: Self::Cached, archetype: &'a Archetype) -> Self::Chunks<'a> {
        panic!()
    }

    fn eval_chunk(chunks: Self::Chunks<'_>, index: usize) -> u64 {
        panic!()
    }
}

impl QueryFilter for Always {
    type Cached = ();
    type Chunks<'a> = ();

    fn prepare() -> Self::Cached {
        ()
    }

    fn eval_archetype(cached: Self::Cached, archetype: &Archetype) -> bool {
        true
    }

    fn cache_chunks<'a>(cached: Self::Cached, archetype: &'a Archetype) -> Self::Chunks<'a> {
        ()
    }

    fn eval_chunk(chunks: Self::Chunks<'_>, index: usize) -> u64 {
        u64::MAX
    }
}

impl QueryFilter for Never {
    type Cached = ();
    type Chunks<'a> = ();

    fn prepare() -> Self::Cached {
        ()
    }

    fn eval_archetype(cached: Self::Cached, archetype: &Archetype) -> bool {
        false
    }

    fn cache_chunks<'a>(cached: Self::Cached, archetype: &'a Archetype) -> Self::Chunks<'a> {
        panic!()
    }

    fn eval_chunk(chunks: Self::Chunks<'_>, index: usize) -> u64 {
        panic!()
    }
}

// Trait implementations for modifiers
impl<A: QueryFilter, B: QueryFilter> QueryFilter for And<A, B> {
    type Cached = (A::Cached, B::Cached);

    fn prepare() -> Self::Cached {
        (A::prepare(), B::prepare())
    }

    fn eval_archetype(cached: &Self::Cached, archetype: &Archetype) -> bool {
        A::eval_archetype(&cached.0, archetype) && B::eval_archetype(&cached.1, archetype)
    }

    fn eval_entry(cached: &Self::Cached, states: StateRow) -> bool {
        A::eval_entry(&cached.0, states) && B::eval_entry(&cached.1, states)
    }
}

impl<A: QueryFilter, B: QueryFilter> QueryFilter for Or<A, B> {
    type Cached = (A::Cached, B::Cached);

    fn prepare() -> Self::Cached {
        (A::prepare(), B::prepare())
    }

    fn eval_archetype(cached: &Self::Cached, archetype: &Archetype) -> bool {
        A::eval_archetype(&cached.0, archetype) || B::eval_archetype(&cached.1, archetype)
    }

    fn eval_entry(cached: &Self::Cached, states: StateRow) -> bool {
        A::eval_entry(&cached.0, states) || B::eval_entry(&cached.1, states)
    }
}

impl<A: QueryFilter, B: QueryFilter> QueryFilter for Xor<A, B> {
    type Cached = (A::Cached, B::Cached);

    fn prepare() -> Self::Cached {
        (A::prepare(), B::prepare())
    }

    fn eval_archetype(cached: &Self::Cached, archetype: &Archetype) -> bool {
        A::eval_archetype(&cached.0, archetype) ^ B::eval_archetype(&cached.1, archetype)
    }

    fn eval_entry(cached: &Self::Cached, states: StateRow) -> bool {
        A::eval_entry(&cached.0, states) ^ B::eval_entry(&cached.1, states)
    }
}

impl<A: QueryFilter> QueryFilter for Not<A> {
    type Cached = A::Cached;

    fn prepare() -> Self::Cached {
        A::prepare()
    }

    fn eval_archetype(cached: &Self::Cached, archetype: &Archetype) -> bool {
        !A::eval_archetype(cached, archetype)
    }

    fn eval_entry(cached: &Self::Cached, states: StateRow) -> bool {
        !A::eval_entry(cached, states)
    }
}
*/

/*
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
*/