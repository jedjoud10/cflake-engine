use crate::prelude::{TexelFormat, Texel, ColorTexel};
use super::{PainterColorLayout, Attachment, ColorAttachmentLayout};
use seq_macro::seq;
use std::concat_idents;

macro_rules! tuple_impls_color_layout {
    ( $( $name:ident )+, $max:tt) => {
        impl<$($name: ColorTexel),+> PainterColorLayout for ($($name,)+) {}
        
    };
}

tuple_impls_color_layout! { C0 C1, 2  }
tuple_impls_color_layout! { C0 C1 C2, 3 }
tuple_impls_color_layout! { C0 C1 C2 C3, 4 }
tuple_impls_color_layout! { C0 C1 C2 C3 C4, 5 }

impl<T: ColorTexel> PainterColorLayout for T {}
impl<'a, T: ColorTexel, A: Attachment<'a, T>> ColorAttachmentLayout<'a, T> for A {
    fn untyped(&self) -> Option<Vec<super::UntypedAttachment>> {
        Attachment::untyped(self).map(|x| vec![x])
    }
}