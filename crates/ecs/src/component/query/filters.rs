use std::marker::PhantomData;
use crate::AccessType;

// Component filters that only let some components through the query
pub struct Changed<T: AccessType>(PhantomData<*const T>);

// TODO: Added / Removed