use std::marker::PhantomData;

use crate::BorrowedItem;

// Component filters that only let some components through the query
pub struct Changed<'a, T: BorrowedItem<'a>>(PhantomData<*const T>, PhantomData<&'a ()>);

// TODO: Added / Removed
