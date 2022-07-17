use crate::{
    registry::{self},
    Component, Mask, StateRow,
};
use std::marker::PhantomData;

// Input data given to the filter
pub struct ItemInput {
    pub(super) state_row: StateRow,
    pub(super) mask: Mask,
}

// Basic evaluator that will be implemented for the filter sources and modifiers
pub trait Evaluate: 'static {
    // Cached data for fast traversal
    type Cached;

    // Create the cache
    fn setup() -> Self::Cached;

    // Evaluate the filter using the proper filter input
    fn eval(cached: &Self::Cached, input: &ItemInput) -> bool;
}

// Filter sources
pub struct Added<T: Component>(PhantomData<T>);
pub struct Modified<T: Component>(PhantomData<T>);
pub struct Contains<T: Component>(PhantomData<T>);

// Constant source that always fail / succeed the test
pub struct Always(());
pub struct Never(());

// Modifiers
pub struct And<A: Evaluate, B: Evaluate>(A, B);
pub struct Or<A: Evaluate, B: Evaluate>(A, B);
pub struct Not<A: Evaluate>(A);

// Trait implementations for sources
impl<T: Component> Evaluate for Added<T> {
    type Cached = Mask;

    fn setup() -> Self::Cached {
        registry::mask::<T>()
    }

    fn eval(cached: &Self::Cached, input: &ItemInput) -> bool {
        input.state_row.was_added_with_offset(cached.offset())
    }
}

impl<T: Component> Evaluate for Modified<T> {
    type Cached = Mask;

    fn setup() -> Self::Cached {
        registry::mask::<T>()
    }

    fn eval(cached: &Self::Cached, input: &ItemInput) -> bool {
        input.state_row.was_mutated_with_offset(cached.offset())
    }
}

impl<T: Component> Evaluate for Contains<T> {
    type Cached = Mask;

    fn setup() -> Self::Cached {
        registry::mask::<T>()
    }

    fn eval(cached: &Self::Cached, input: &ItemInput) -> bool {
        input.mask.contains(*cached)
    }
}

impl Evaluate for Always {
    type Cached = ();

    fn setup() -> Self::Cached {}

    fn eval(_cached: &Self::Cached, _input: &ItemInput) -> bool {
        true
    }
}

impl Evaluate for Never {
    type Cached = ();

    fn setup() -> Self::Cached {}

    fn eval(_cached: &Self::Cached, _input: &ItemInput) -> bool {
        false
    }
}

// Trait implementations for modifiers
impl<A: Evaluate, B: Evaluate> Evaluate for And<A, B> {
    type Cached = (A::Cached, B::Cached);

    fn setup() -> Self::Cached {
        (A::setup(), B::setup())
    }

    fn eval(cached: &Self::Cached, input: &ItemInput) -> bool {
        A::eval(&cached.0, input) && B::eval(&cached.1, input)
    }
}

impl<A: Evaluate, B: Evaluate> Evaluate for Or<A, B> {
    type Cached = (A::Cached, B::Cached);

    fn setup() -> Self::Cached {
        (A::setup(), B::setup())
    }

    fn eval(cached: &Self::Cached, input: &ItemInput) -> bool {
        A::eval(&cached.0, input) || B::eval(&cached.1, input)
    }
}

impl<A: Evaluate> Evaluate for Not<A> {
    type Cached = A::Cached;

    fn setup() -> Self::Cached {
        A::setup()
    }

    fn eval(cached: &Self::Cached, input: &ItemInput) -> bool {
        !A::eval(cached, input)
    }
}

// Functions to create the sources and modifiers
pub fn modified<T: Component>() -> Modified<T> {
    Modified(PhantomData::default())
}
pub fn added<T: Component>() -> Added<T> {
    Added(PhantomData::default())
}
pub fn contains<T: Component>() -> Contains<T> {
    Contains(PhantomData::default())
}

// Constant sources
pub fn always() -> Always {
    Always(())
}
pub fn never() -> Never {
    Never(())
}

// Modifiers
pub fn and<A: Evaluate, B: Evaluate>(a: A, b: B) -> And<A, B> {
    And(a, b)
}
pub fn or<A: Evaluate, B: Evaluate>(a: A, b: B) -> Or<A, B> {
    Or(a, b)
}
pub fn not<A: Evaluate>(a: A) -> Not<A> {
    Not(a)
}
