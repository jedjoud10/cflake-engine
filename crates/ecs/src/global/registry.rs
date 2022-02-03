use std::any::Any;

use super::Global;
use crate::utils::GlobalError;

// Cast a boxed global to a reference of that global
pub(crate) fn cast_global<'a, T>(global: &'a dyn Global) -> Result<&T, GlobalError>
where
    T: Global + 'static,
{
    let component_any: &dyn Any = global.as_any();
    let reference = component_any.downcast_ref::<T>().ok_or_else(|| GlobalError::new("Could not cast component".to_string()))?;
    Ok(reference)
}
// Cast a boxed global to a mutable reference of that global
pub(crate) fn cast_global_mut<'a, T>(linked_component: &'a mut dyn Global) -> Result<&mut T, GlobalError>
where
    T: Global + 'static,
{
    let component_any: &mut dyn Any = linked_component.as_any_mut();
    let reference_mut = component_any.downcast_mut::<T>().ok_or_else(|| GlobalError::new("Could not cast component".to_string()))?;
    Ok(reference_mut)
}
