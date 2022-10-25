use super::ThreadFuncEntry;
use crate::SliceTuple;
use std::{marker::PhantomData, sync::Arc};

// This is a clonable typed function that can be used by the user if they wish to
pub struct TypedClonableFn<'a, S: for<'i> SliceTuple<'i>>(
    Arc<dyn Fn(<S as SliceTuple<'_>>::ItemTuple) + Send + Sync + 'a>,
    Arc<dyn Fn(ThreadFuncEntry) + Send + Sync + 'a>,
    PhantomData<S>,
);
unsafe impl<'a, S: for<'i> SliceTuple<'i>> Sync for TypedClonableFn<'a, S> {}
unsafe impl<'a, S: for<'i> SliceTuple<'i>> Send for TypedClonableFn<'a, S> {}
impl<'a, S: for<'i> SliceTuple<'i>> Clone for TypedClonableFn<'a, S> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone(), self.2.clone())
    }
}

// This is a clonable untyped function. Only used internally
pub(super) struct UntypedClonableFn<'a>(pub Arc<dyn Fn(ThreadFuncEntry) + Send + Sync + 'a>);
impl<'a> Clone for UntypedClonableFn<'a> {
    fn clone(&self) -> Self {
        Self(<Arc<dyn Fn(ThreadFuncEntry) + Send + Sync + 'a> as Clone>::clone(&self.0))
    }
}

// Trait implemented for Arc and Fn implementations of the specfiic functions that take in L::ItemTuple
// I implemented this for the s
pub trait ForEachFunction<'a, S: for<'i> SliceTuple<'i>>: Send + Sync + 'a {
    fn execute(&self, item: <S as SliceTuple<'_>>::ItemTuple);
    fn into_clonable_fn(self) -> TypedClonableFn<'a, S>;
}

impl<'a, S: for<'i> SliceTuple<'i>, F: Fn(<S as SliceTuple<'_>>::ItemTuple) + Send + Sync + 'a>
    ForEachFunction<'a, S> for F
{
    fn execute(&self, item: <S as SliceTuple<'_>>::ItemTuple) {
        self(item);
    }

    fn into_clonable_fn(self) -> TypedClonableFn<'a, S> {
        type AltArcFn<'b> = Arc<dyn Fn(ThreadFuncEntry) + Send + Sync + 'b>;

        let main = Arc::new(move |item: <S as SliceTuple<'_>>::ItemTuple| self(item));
        let cloned = main.clone();

        let function: AltArcFn<'a> = Arc::new(move |entry: ThreadFuncEntry| unsafe {
            let cloned = &*cloned.clone();
            let offset = entry.batch_offset;
            let ptrs = entry.base.downcast::<S::PtrTuple>().ok();
            let mut ptrs = ptrs
                .map(|ptrs| S::from_ptrs(&*ptrs, entry.batch_length, offset))
                .unwrap();

            for i in 0..entry.batch_length {
                cloned(S::get_unchecked(&mut ptrs, i));
            }
        });

        TypedClonableFn(main, function, PhantomData)
    }
}

impl<'a, S: for<'i> SliceTuple<'i> + 'a> ForEachFunction<'a, S> for TypedClonableFn<'a, S> {
    fn execute(&self, item: <S as SliceTuple<'_>>::ItemTuple) {
        self.0(item);
    }

    fn into_clonable_fn(self) -> TypedClonableFn<'a, S> {
        self
    }
}
