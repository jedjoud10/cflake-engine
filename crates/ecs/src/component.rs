use std::any::Any;

// The main component trait
// We do a little bit of googling https://stackoverflow.com/questions/26983355/is-there-a-way-to-combine-multiple-traits-in-order-to-define-a-new-trait
pub trait Component: ComponentInternal + Sync + Send where Self: Sized {
    // Wrappers
    fn get_component_id() -> usize {
        crate::registry::get_component_id::<Self>()
    }
    fn is_registered() -> bool {
        crate::registry::is_component_registered::<Self>()
    }
}
// A component trait that can be added to other components
pub trait ComponentInternal {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_component_name() -> String where Self: Sized;  
}