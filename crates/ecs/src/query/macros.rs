use crate::{
    mask, Archetype, Component, ComponentTable, LayoutAccess, LinkError, Mask, MaskMap, OwnedBundle, OwnedBundleTableAccessor
};

use seq_macro::seq;
use casey::lower;

macro_rules! tuple_impls {
    ( $( $name:ident )+, $max:tt ) => {
        // Implement the owned bundle for component sets
        impl<'a, $($name: Component),+> OwnedBundle<'a> for ($($name,)+) {
            type Storages = ($(&'a mut Vec<$name>),+);

            fn combined() -> Mask {
                ($(mask::<$name>())|+)
            }

            fn is_valid() -> bool {
                let anded = ($(mask::<$name>())&+);
                anded == Mask::zero()
            }

            fn fetch(archetype: &'a mut Archetype) -> Self::Storages {
                todo!()
                //assert!(Self::is_valid());
            }

            fn push(storages: &mut Self::Storages, bundle: Self) {

            }
        }

        // Implement the owned bundle table accessor for component sets
        impl<$($name: Component),+> OwnedBundleTableAccessor for ($($name,)+) {
            fn default_tables() -> MaskMap<Box<dyn ComponentTable>> {
                let mut map = MaskMap::<Box<dyn ComponentTable>>::default();
                ($(
                    map.insert(mask::<$name>(), Box::new(Vec::<$name>::new()))
                ),+);
                map
            }

            fn swap_remove(tables: &mut MaskMap<Box<dyn ComponentTable>>, index: usize) -> Self {
                todo!()
            }

            fn push(storages: &mut MaskMap<Box<dyn ComponentTable>>, bundle: Self) -> Self {
                todo!()
            }
        }
    };
}

tuple_impls! { C0, 1 }