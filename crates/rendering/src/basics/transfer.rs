// A simple wrapper that just indicates that the inner value will be cloned (Arc-style) and sent to the pipeline, but it'll stay on the main thread so we can write to it
pub struct Transfer<T: Clone>(pub(crate) T);

pub trait Transferable: Clone {
    fn transfer(&self) -> Transfer<Self>;
} 