use crate::{QueryLayout, QueryItem, LayoutAccess, Archetype, OwnedBundle, Component, Mask, mask, name, ComponentTable, MaskHashMap, };
use casey::lower;
use seq_macro::seq;

macro_rules! tuple_impls {
    ( $( $name:ident )+, $max:tt ) => {       
        impl<'a, $($name: Component),+> OwnedBundle<'a> for ($($name,)+) {
            type Storages = ($(&'a mut Vec<$name>),+);

            fn items() -> usize {
                $max
            }
        
            fn name(index: usize) -> Option<&'static str> {
                let names = [$(name::<$name>()),+];
                names.get(index).cloned()
            }
            
            fn mask(index: usize) -> Option<Mask> {
                let masks = [$(mask::<$name>()),+];
                masks.get(index).cloned()
            }
        
            fn reduce(mut lambda: impl FnMut(Mask, Mask) -> Mask) -> Mask {
                let masks = [$(mask::<$name>()),+];
                masks[..].into_iter().cloned().reduce(|a, b| lambda(a, b)).unwrap()
            }

            fn prepare(archetype: &'a mut Archetype) -> Option<Self::Storages> {
                assert!(Self::is_valid());
                seq!(N in 0..$max {
                    let table = archetype.table_mut::<C~N>()?;
                    let ptr = table as *mut Vec<C~N>;
                    let c~N = unsafe { &mut *ptr };
                });

                Some(($(
                    lower!($name)
                ),+,))
            }

            fn push(storages: &mut Self::Storages, bundle: Self) {
                seq!(N in 0..$max {
                    let vec = &mut storages.N;
                    vec.push(bundle.N);
                });
            }

            fn default_tables() -> MaskHashMap<Box<dyn ComponentTable>> {
                let mut map = MaskHashMap::<Box<dyn ComponentTable>>::default();
                ($(
                    map.insert(mask::<$name>(), Box::new(Vec::<$name>::new()))
                ),+);
                map
            }

            fn try_swap_remove(tables: &mut MaskHashMap<Box<dyn ComponentTable>>, index: usize) -> Option<Self> {
                seq!(N in 0..$max {
                    let boxed = tables.get_mut(&mask::<C~N>())?;
                    let vec = boxed.as_any_mut().downcast_mut::<Vec<C~N>>().unwrap();
                    let c~N: C~N = vec.swap_remove(index);
                });

                Some(($(
                    lower!($name)
                ),+,))
            }
        }
        
        impl<'s: 'i, 'i, $($name: QueryItem<'s, 'i>, )+> QueryLayout<'s, 'i> for ($($name,)+) {
            type SliceTuple = ($($name::Slice,)+);
        
            fn items() -> usize {
                $max
            }
        
            fn name(index: usize) -> Option<&'static str> {
                let names = [$($name::name()),+];
                names.get(index).cloned()
            }
            
            fn access(index: usize) -> Option<LayoutAccess> {
                let accesses = [$($name::access()),+];
                accesses.get(index).cloned()
            }
        
            fn reduce(mut lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess {
                let layouts = [$($name::access()),+];
                layouts[..].into_iter().cloned().reduce(|a, b| lambda(a, b)).unwrap()
            }

            unsafe fn slices_from_archetype_unchecked(archetype: &Archetype) -> Self::SliceTuple {
                seq!(N in 0..$max {
                    let ptr~N = C~N::ptr_from_archetype_unchecked(archetype);
                    let c~N = C~N::from_raw_parts(ptr~N, archetype.len());
                });

                ($(
                    lower!($name)
                ),+,)
            }
        
            unsafe fn slices_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::SliceTuple {
                seq!(N in 0..$max {
                    let ptr~N = C~N::ptr_from_mut_archetype_unchecked(archetype);
                    let c~N = C~N::from_raw_parts(ptr~N, archetype.len());
                });

                ($(
                    lower!($name)
                ),+,)
            }
        
            unsafe fn get_unchecked(slice: Self::SliceTuple, index: usize) -> Self {
                seq!(N in 0..$max {
                    let c~N = C~N::get_unchecked(slice.N, index);
                });

                ($(
                    lower!($name)
                ),+,)
            }
        }
    };
}

tuple_impls! { C0 C1, 2 }
tuple_impls! { C0 C1 C2, 3 }
tuple_impls! { C0 C1 C2 C3, 4 }
tuple_impls! { C0 C1 C2 C3 C4, 5 }
tuple_impls! { C0 C1 C2 C3 C4 C5, 6 }
tuple_impls! { C0 C1 C2 C3 C4 C5 C6, 7 }
tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7, 8 }
tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8, 9 }
tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9, 10 }