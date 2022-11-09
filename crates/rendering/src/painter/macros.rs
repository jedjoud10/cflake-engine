use super::{AsTarget, ColorTupleTargets, MaybeColorLayout, UntypedTarget};
use crate::prelude::ColorTexel;
use seq_macro::seq;
use paste::paste;

macro_rules! tuple_impls_color_layout {
    ( $( $name:ident )+, $max:tt, $( $name2:ident )+) => {
        paste! {
            impl<$($name: ColorTexel),+> MaybeColorLayout for ($($name,)+) {}
            impl<$($name: ColorTexel),+, $($name2: AsTarget<[< T $name2 >]>),+> ColorTupleTargets<($($name),+)> for ($($name2),+) {
                fn untyped_targets(self) -> Option<Vec<UntypedTarget>> {
                    let mut vec = Vec::with_capacity($max);
                    seq!(N in 0..$max {
                        vec.push(AsTarget::as_untyped_target(self.N).unwrap());
                    });
                    Some(vec)
                }
            }
        }
    }
}

// TODO: Fix this lil hack bozo
tuple_impls_color_layout! { TA0 TA1, 2, A0 A1  }
tuple_impls_color_layout! { TA0 TA1 TA2, 3, A0 A1 A2 }
tuple_impls_color_layout! { TA0 TA1 TA2 TA3, 4, A0 A1 A2 A3 }
tuple_impls_color_layout! { TA0 TA1 TA2 TA3 TA4, 5, A0 A1 A2 A3 A4 }

impl<T: ColorTexel> MaybeColorLayout for T {}

impl<CT: ColorTexel, T: AsTarget<CT>> ColorTupleTargets<CT> for T {
    fn untyped_targets(self) -> Option<Vec<UntypedTarget>> {
        Some(vec![self.as_untyped_target().unwrap()])
    }
}

impl<'a> ColorTupleTargets<()> for () {
    fn untyped_targets(self) -> Option<Vec<UntypedTarget>> {
        None
    }
}
