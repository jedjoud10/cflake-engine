use crate::AccessType;
use std::marker::PhantomData;

// Component filters that only let some components through the query
pub struct Changed<T: AccessType>(PhantomData<*const T>);

// TODO: Added / Removed
