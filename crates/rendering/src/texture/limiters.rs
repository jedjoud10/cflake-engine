use std::marker::PhantomData;
use super::{Element, Base};

// A range texel limiter that will hint the texture that the integer must be accessed as a floating point value, and that it must be in the 0-1 range
pub struct Ranged<T: Base>(PhantomData<T>);

// A normalized texel limiter that will the texture that the integer must be accessed as a floating point value, and that it must be in the -1 - 1 range
pub struct Normalized<T: Base>(PhantomData<T>);

impl<T: Base> Element for Ranged<T> {}
impl<T: Base> Element for Normalized<T> {}