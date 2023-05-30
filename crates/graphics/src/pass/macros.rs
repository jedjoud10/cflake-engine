use crate::{ColorTexel, Texel, TexelInfo, ColorLayout, BlendState};
use seq_macro::seq;

macro_rules! impl_color_layout {
    ( $( $name:ident )+, $max:tt ) => {
        impl<$($name: ColorTexel),+> ColorLayout for ($($name,)+) {
            type BlendingArray = [Option<BlendState>; $max];

            fn layout_info() -> Vec<TexelInfo> {
                let mut vec = Vec::with_capacity($max);

                seq!(N in 0..$max {
                    vec.push(<C~N as Texel>::info());
                });

                vec
            }
        }
    };
}

impl_color_layout! { C0 C1, 2 }
impl_color_layout! { C0 C1 C2, 3 }
impl_color_layout! { C0 C1 C2 C3, 4 }
impl_color_layout! { C0 C1 C2 C3 C4, 5 }
impl_color_layout! { C0 C1 C2 C3 C4 C5, 6 }