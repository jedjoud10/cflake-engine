use super::{CanvasLayout, AttachmentDescription};
use crate::{texture::{Texture2D, Depth, Ranged}, prelude::{Texel, Texture}};
use seq_macro::seq;

impl<T: Texel> CanvasLayout for Texture2D<T> {
    fn resize(&mut self, size: vek::Extent2<u16>) {
        <Self as Texture>::resize(self, size);
    }

    fn attachments(&self) -> Vec<AttachmentDescription> {
        vec![AttachmentDescription::new(self)]
    }
}

impl CanvasLayout for () {
    fn resize(&mut self, size: vek::Extent2<u16>) {}

    fn attachments(&self) -> Vec<AttachmentDescription> {
        Vec::new()
    }

    fn is_instantiable(&self) -> bool {
        false
    }
}

macro_rules! tuple_impls_color_layout {
    ( $( $name:ident )+, $max:tt) => {
        impl<$($name: Texel),+> CanvasLayout for ($(Texture2D<$name>,)+) {
            fn resize(&mut self, size: vek::Extent2<u16>) {
                seq!(N in 0..$max {
                    Texture::resize(&mut self.N, size);
                });
            }

            fn attachments(&self) -> Vec<AttachmentDescription> {
                let mut vec = Vec::with_capacity($max);
                
                seq!(N in 0..$max {
                    vec.push(AttachmentDescription::new(&self.N));
                });

                vec
            }
        }
    };
}

tuple_impls_color_layout! { C0 C1, 2 }
tuple_impls_color_layout! { C0 C1 C2, 3 }
tuple_impls_color_layout! { C0 C1 C2 C3, 4 }
tuple_impls_color_layout! { C0 C1 C2 C3 C4, 5 }