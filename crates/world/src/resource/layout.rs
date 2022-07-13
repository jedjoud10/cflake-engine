use ahash::AHashSet;
use utils::GenericReference;
use std::{
    any::{type_name, TypeId},
    ptr::NonNull,
};

use crate::{Resource, ResourceError, World};

// We store the type ID and name in their own struct since the handle might not even be mutable
pub struct ResourceReferenceDesc {
    _type: TypeId,
    name: &'static str,
    mutable: bool,
};

// A layout simply multiple resource handles of different resources
pub trait Layout<'a>: Sized {
    // Get a list of the Handle IDs of the underlying resources
    fn descriptions() -> Vec<ResourceReferenceDesc>;

    // Check if the layout is valid (no intersecting handles)
    fn validate() -> Result<(), ResourceError> {
        let types = Self::descriptions();
        let mut map = AHashSet::new();
        let name = types
            .iter()
            .find(|ResourceReferenceDesc { _type, name, mutable }| !map.insert(_type) && *mutable);

        // This is a certified inversion classic
        if let Some((_, name, _)) = name {
            Err(ResourceError::Overlapping(name))
        } else {
            Ok(())
        }
    }

    // Get the layout tuple from the resource world without actually checking if the layout is valid
    unsafe fn fetch_unchecked(world: &'a mut World) -> Result<Self, ResourceError>;
}

// Get the handle ID of a resource generic reference 
fn id<'a, A: GenericReference<'a>>() -> ResourceReferenceDesc where A::Inner: Resource {
    (
        TypeId::of::<A::Inner>(),
        type_name::<A::Inner>(),
        A::MUTABLE,
    )
}

// Simple wrapping function that just gets the handle from the world, and makes it so the lifetime of the handle is different than the one of the world
unsafe fn fetch<'a, A: GenericReference<'a>>(world: &mut World) -> Result<A, ResourceError> where A::Inner: Resource {
    let ptr = <A::Inner as Resource>::fetch_ptr(world);
    let value = ptr.map(|ptr| A::_as_from_mut_ptr(ptr.as_ptr()));
    value
}

impl<'a, A: GenericReference<'a>> Layout<'a> for A where A::Inner: Resource {
    fn descriptions() -> Vec<ResourceReferenceDesc> {
        vec![id::<A>()]
    }

    unsafe fn fetch_unchecked(world: &'a mut World) -> Result<Self, ResourceError> {
        fetch(world)
    }
}

macro_rules! tuple_impls {
    ( $( $name:ident )+ ) => {
        impl<'a, $($name: Bruh<'a>),+> Testio for ($($name,)+) where $($name::Inner: Testio),+
        {
            fn do_something(self) -> Self {
                //let ($($name,)+) = self;
                //($($name.do_something(),)+)
                self
            }
        }
    };
}