use ahash::AHashSet;
use std::{
    any::{type_name, TypeId},
    ptr::NonNull,
};

use crate::{Resource, ResourceError, World, ResourceReferenceDesc, ResourceReference};

// A layout simply multiple resource handles of different resources
pub trait ResourceLayout<'a>: Sized {
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
        if let Some(ResourceReferenceDesc { name, .. }) = name {
            Err(ResourceError::Overlapping(name))
        } else {
            Ok(())
        }
    }

    // Get the layout tuple from the resource world without actually checking if the layout is valid
    unsafe fn fetch_unchecked(world: &'a mut World) -> Result<Self, ResourceError>;
}

// Simple wrapping function that just gets the handle from the world, and makes it so the lifetime of the handle is different than the one of the world
unsafe fn fetch<'a, A: ResourceReference<'a>>(world: &mut World) -> Result<A, ResourceError> {
    let ptr = <A::Inner as Resource>::fetch_ptr(world);
    let value = ptr.map(|ptr| A::from_non_null(ptr));
    value
}

impl<'a, A: ResourceReference<'a>> ResourceLayout<'a> for A {
    fn descriptions() -> Vec<ResourceReferenceDesc> {
        vec![A::descriptor()]
    }

    unsafe fn fetch_unchecked(world: &'a mut World) -> Result<Self, ResourceError> {
        fetch(world)
    }
}

macro_rules! tuple_impls {
    ( $( $name:ident )+ ) => {
        impl<'a, $($name: ResourceReference<'a>),+> Layout<'a> for ($($name,)+)
        {
            fn descriptions() -> Vec<ResourceReferenceDesc> {
                vec![$($name::descriptor()),+]
            }
        
            unsafe fn fetch_unchecked(world: &'a mut World) -> Result<Self, ResourceError> {
                let data = ($(fetch::<$name>(world)?,)+);
                Ok(data)
            }
        }
    };
}

// Heheheha
tuple_impls! { A }
tuple_impls! { A B }
tuple_impls! { A B C }
tuple_impls! { A B C D }
tuple_impls! { A B C D E }
tuple_impls! { A B C D E F }
tuple_impls! { A B C D E F G }
tuple_impls! { A B C D E F G H }
tuple_impls! { A B C D E F G H I }
tuple_impls! { A B C D E F G H I J }
tuple_impls! { A B C D E F G H I J K }
tuple_impls! { A B C D E F G H I J K L }
tuple_impls! { A B C D E F G H I J K L M }
tuple_impls! { A B C D E F G H I J K L M N }
tuple_impls! { A B C D E F G H I J K L M N O }