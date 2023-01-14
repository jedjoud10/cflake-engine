use std::marker::PhantomData;

// These are the bindings that will be passed to each materials whenever we want to bind descriptor sets
// These bindings will check whenever you updated some data or not
pub struct Bindings<'a> {
    _phantom: PhantomData<&'a ()>,
}