use ahash::AHashSet;


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
            .find(|ResourceReferenceDesc { _type, name: _, mutable }| !map.insert(_type) && *mutable);

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
    
    ptr.map(|ptr| A::from_non_null(ptr))
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
        // Implement the resource layout for all tuple types that contain resource references
        impl<'a, $($name: ResourceReference<'a>),+> ResourceLayout<'a> for ($($name,)+)
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
tuple_impls! { R1 R2 }
tuple_impls! { R1 R2 R3 }
tuple_impls! { R1 R2 R3 R4 }
tuple_impls! { R1 R2 R3 R4 R5 }
tuple_impls! { R1 R2 R3 R4 R5 R6 }
tuple_impls! { R1 R2 R3 R4 R5 R6 R7 }
tuple_impls! { R1 R2 R3 R4 R5 R6 R7 R8 }
tuple_impls! { R1 R2 R3 R4 R5 R6 R7 R8 R9 }
tuple_impls! { R1 R2 R3 R4 R5 R6 R7 R8 R9 R10 }
tuple_impls! { R1 R2 R3 R4 R5 R6 R7 R8 R9 R10 R11 }
tuple_impls! { R1 R2 R3 R4 R5 R6 R7 R8 R9 R10 R11 R12 }
tuple_impls! { R1 R2 R3 R4 R5 R6 R7 R8 R9 R10 R11 R12 R13 }
tuple_impls! { R1 R2 R3 R4 R5 R6 R7 R8 R9 R10 R11 R12 R13 R14 }
tuple_impls! { R1 R2 R3 R4 R5 R6 R7 R8 R9 R10 R11 R12 R13 R14 R15 }