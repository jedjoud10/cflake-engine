use crate::error::GlobalError;
use crate::global::Global;
use std::any::Any;

// Cast a boxed global to a reference of that global
pub(crate) fn cast_global<T>(global: &dyn Global) -> Result<&T, GlobalError>
where
    T: Global + 'static,
{
    let component_any: &dyn Any = global.as_any();
    let reference = component_any.downcast_ref::<T>().ok_or_else(|| GlobalError::new("Could not cast global!".to_string()))?;
    Ok(reference)
}
// Cast a boxed global to a mutable reference of that global
pub(crate) fn cast_global_mut<T>(linked_component: &mut dyn Global) -> Result<&mut T, GlobalError>
where
    T: Global + 'static,
{
    let component_any: &mut dyn Any = linked_component.as_any_mut();
    let reference_mut = component_any.downcast_mut::<T>().ok_or_else(|| GlobalError::new("Could not cast global!".to_string()))?;
    Ok(reference_mut)
}
