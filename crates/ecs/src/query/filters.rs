use std::marker::PhantomData;

use crate::ComponentBorrower;

// Component filters that only let some components through the query
pub struct Changed<'a, T: ComponentBorrower<'a>>(PhantomData<*const T>, PhantomData<&'a ()>);

// TODO: Added / Removed
