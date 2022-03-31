// Implemented for components
pub trait Component
where
    Self: 'static + Sync + Send,
{
}

impl<T> Component for T where T: 'static + Sized + Send + Sync {}
