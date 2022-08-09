use super::{CanvasLayout, ColorCanvasAttachment};
use crate::texture::{ColorTexel, DepthTexel, StencilTexel, Texture2D};

macro_rules! tuple_impls {
    ( $( $name:ident )+) => {
        impl<$($name: ColorCanvasAttachment),+> CanvasLayout for ($($name,)+) {
            fn resize(&mut self, size: vek::Extent2<u16>) {

            }
        }

        impl<$($name: ColorCanvasAttachment),+, D: DepthTexel> CanvasLayout for ($($name,)+ Texture2D<D>) {
            fn resize(&mut self, size: vek::Extent2<u16>) {

            }
        }

        impl<$($name: ColorCanvasAttachment),+, S: StencilTexel> CanvasLayout for ($($name,)+ Texture2D<S>) {
            fn resize(&mut self, size: vek::Extent2<u16>) {

            }
        }

        impl<$($name: ColorCanvasAttachment),+, D: DepthTexel, S: StencilTexel> CanvasLayout for ($($name,)+ Texture2D<S>, Texture2D<D>) {
            fn resize(&mut self, size: vek::Extent2<u16>) {

            }
        }
    };
}

tuple_impls! { C0 C1 }
tuple_impls! { C0 C1 C2 }
tuple_impls! { C0 C1 C2 C3 }
