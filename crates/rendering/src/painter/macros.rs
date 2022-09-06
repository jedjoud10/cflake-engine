use crate::prelude::{TexelFormat, Texel, ColorTexel};
use super::{PainterColorLayout, Attachment, ColorAttachmentLayout};
use seq_macro::seq;
use std::concat_idents;

macro_rules! tuple_impls_color_layout {
    ( $( $name:ident )+, $max:tt, $( $name2:ident )+) => {
        impl<$($name: ColorTexel),+> PainterColorLayout for ($($name,)+) {}
        impl<'a, $($name: ColorTexel),+, $($name2: Attachment<'a, concat_idents!($name2)>),+> ColorAttachmentLayout<'a, ($($name),+)> for ($($name2),+) {
            fn untyped(&self) -> Option<Vec<super::UntypedAttachment>> {
                let mut vec = Vec::with_capacity($max);
                seq!(N in 0..$max {
                    vec.push(Attachment::untyped(&self.N).unwrap());
                });
                Some(vec)
            }
        }
    }
}

// TODO: Fix this lil hack bozo
tuple_impls_color_layout! { AT0 AT1, 2, A0 A1  }
tuple_impls_color_layout! { AT0 AT1 AT2, 3, A0 A1 A2 }
tuple_impls_color_layout! { AT0 AT1 AT2 AT3, 4, A0 A1 A2 A3 }
tuple_impls_color_layout! { AT0 AT1 AT2 AT3 AT4, 5, A0 A1 A2 A3 A4 }

impl<T: ColorTexel> PainterColorLayout for T {}
impl<'a, T: ColorTexel, A: Attachment<'a, T>> ColorAttachmentLayout<'a, T> for A {
    fn untyped(&self) -> Option<Vec<super::UntypedAttachment>> {
        Attachment::untyped(self).map(|x| vec![x])
    }
}