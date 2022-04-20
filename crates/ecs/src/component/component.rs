pub use ecs_derive::Component;
// Implemented for components
pub trait Component
where
    Self: 'static + Sync + Send,
{
}
