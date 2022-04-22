use crate::{
    registry::{self},
    Component, ComponentStateRow, Mask, QueryLayout,
};
use std::marker::PhantomData;

// Input data given to the filter
pub struct Input {
    pub(super) row: ComponentStateRow,
}

// Basic evaluator that will be implemented for the filter sources and modifiers
pub trait Evaluate: 'static {
    // Cached data for fast traversal
    type Cached;

    // Create the cache
    fn setup() -> Self::Cached;

    // Evaluate the filter using the proper filter input
    fn eval(cached: &Self::Cached, input: &Input) -> bool;
}

// Sources
pub struct Added<T: Component>(PhantomData<T>);
pub struct Modified<T: Component>(PhantomData<T>);

// Constant source that always fails / succeeds the test
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

    fn eval(cached: &Self::Cached, input: &Input) -> bool {
        input.row.added(cached.offset())
    }
}

impl<T: Component> Evaluate for Modified<T> {
    type Cached = Mask;

    fn setup() -> Self::Cached {
        registry::mask::<T>()
    }

    fn eval(cached: &Self::Cached, input: &Input) -> bool {
        input.row.mutated(cached.offset())
    }
}

impl Evaluate for Always {
    type Cached = ();

    fn setup() -> Self::Cached {}

    fn eval(_cached: &Self::Cached, _input: &Input) -> bool {
        true
    }
}

impl Evaluate for Never {
    type Cached = ();

    fn setup() -> Self::Cached {}

    fn eval(_cached: &Self::Cached, _input: &Input) -> bool {
        false
    }
}

// Trait implementations for modifiers
impl<A: Evaluate, B: Evaluate> Evaluate for And<A, B> {
    type Cached = (A::Cached, B::Cached);

    fn setup() -> Self::Cached {
        (A::setup(), B::setup())
    }

    fn eval(cached: &Self::Cached, input: &Input) -> bool {
        A::eval(&cached.0, input) && B::eval(&cached.1, input)
    }
}

impl<A: Evaluate, B: Evaluate> Evaluate for Or<A, B> {
    type Cached = (A::Cached, B::Cached);

    fn setup() -> Self::Cached {
        (A::setup(), B::setup())
    }

    fn eval(cached: &Self::Cached, input: &Input) -> bool {
        A::eval(&cached.0, input) || B::eval(&cached.1, input)
    }
}

impl<A: Evaluate> Evaluate for Not<A> {
    type Cached = A::Cached;

    fn setup() -> Self::Cached {
        A::setup()
    }

    fn eval(cached: &Self::Cached, input: &Input) -> bool {
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

// Constant sources
pub const fn always() -> Always {
    Always(())
}
pub const fn never() -> Never {
    Never(())
}

// Modifiers
pub const fn and<A: Evaluate, B: Evaluate>(a: A, b: B) -> And<A, B> {
    And(a, b)
}
pub const fn or<A: Evaluate, B: Evaluate>(a: A, b: B) -> Or<A, B> {
    Or(a, b)
}
pub const fn not<A: Evaluate>(a: A) -> Not<A> {
    Not(a)
}
