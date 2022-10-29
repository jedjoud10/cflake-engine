use crate::{
    registry::{self},
    Component, Mask, StateRow, Archetype,
};
use std::{marker::PhantomData, ops::BitAnd};

// Basic evaluator that will be implemented for the filter sources and modifiers
// These filters allow users to discard certain entries when iterating
pub trait QueryFilter: 'static {
    // Cached data for fast traversal
    type Cached;
    
    // Caching, archetype deletion, and entry evaluation
    fn prepare() -> Self::Cached;
    fn eval_archetype(_: &Self::Cached, _: &Archetype) -> bool { true }
    fn eval_entry(cached: &Self::Cached, states: StateRow) -> bool;
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

    fn prepare() -> Self::Cached {
        registry::mask::<T>()
    }

    fn eval_entry(cached: &Self::Cached, states: StateRow) -> bool {
        states.added().get(cached.offset())
    }
}

impl<T: Component> QueryFilter for Removed<T> {
    type Cached = Mask;

    fn prepare() -> Self::Cached {
        registry::mask::<T>()
    }

    fn eval_entry(cached: &Self::Cached, states: StateRow) -> bool {
        states.removed().get(cached.offset())
    }
}

impl<T: Component> QueryFilter for Modified<T> {
    type Cached = Mask;

    fn prepare() -> Self::Cached {
        registry::mask::<T>()
    }

    fn eval_entry(cached: &Self::Cached, states: StateRow) -> bool {
        states.mutated().get(cached.offset())
    }
}

impl<T: Component> QueryFilter for Contains<T> {
    type Cached = Mask;

    fn prepare() -> Self::Cached {
        registry::mask::<T>()
    }


    fn eval_archetype(cached: &Self::Cached, archetype: &Archetype) -> bool {
        archetype.mask().contains(*cached)
    }

    fn eval_entry(_: &Self::Cached, _states: StateRow) -> bool {
        true
    }
}

impl QueryFilter for Always {
    type Cached = ();

    fn prepare() -> Self::Cached {}
    fn eval_entry(_cached: &Self::Cached, _states: StateRow) -> bool {
        true
    }
}

impl QueryFilter for Never {
    type Cached = ();

    fn prepare() -> Self::Cached {}
    fn eval_entry(_cached: &Self::Cached, _states: StateRow) -> bool {
        false
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

// Functions to create the sources and modifiers
pub fn modified<T: Component>() -> Wrap<Modified<T>> {
    Wrap::<Modified::<T>>(PhantomData)
}
pub fn removed<T: Component>() -> Wrap<Removed<T>> {
    Wrap::<Removed::<T>>(PhantomData)
}
pub fn added<T: Component>() -> Wrap<Added<T>> {
    Wrap::<Added::<T>>(PhantomData)
}
pub fn contains<T: Component>() -> Wrap<Contains<T>> {
    Wrap::<Contains::<T>>(PhantomData)
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