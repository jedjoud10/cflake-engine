use crate::{registry, Archetype, Component, ComponentError, Entity, Mask, QueryCache};

// Something that can be queried. This will be implement on Read<T> and Write<T> (where T is Component). This will also be implemented on Read<Entity>
pub trait QueryItem<'a> {
    // The item type (either T: Component or Entity)
    type Item: 'static;

    // The safe referenced item that user accesses
    type BorrowedItem: 'a;

    // Try to get the mask of this query item (this might fail since Read<Entity> doesn't have a mask)
    fn item_mask() -> Result<Mask, ComponentError>;
}

impl<'a, T: Component> QueryItem<'a> for crate::Read<T> {
    type Item = T;
    type BorrowedItem = &'a T;
    fn item_mask() -> Result<Mask, ComponentError> {
        registry::mask::<T>()
    }
}

impl<'a, T: Component> QueryItem<'a> for crate::Write< T> {
    type Item = T;
    type BorrowedItem = &'a mut T;
    fn item_mask() -> Result<Mask, ComponentError> {
        registry::mask::<T>()
    }
}

impl<'a> QueryItem<'a> for crate::Read<Entity> {
    type Item = Entity;
    type BorrowedItem = &'a Entity;
    fn item_mask() -> Result<Mask, ComponentError> {
        Ok(Mask::default())
    }
}
