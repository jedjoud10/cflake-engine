use super::{CanvasLayout, CanvasColorLayout, CanvasSpecialLayout, AttachmentDescription};
use crate::{texture::{ColorTexel, DepthTexel, StencilTexel, StencilOrDepthTexel, Texture2D, Depth, Ranged}, prelude::{Texel, Texture}};
use seq_macro::seq;

impl<T: ColorTexel> CanvasColorLayout for Texture2D<T> {
    fn resize(&mut self, size: vek::Extent2<u16>) {
        <Self as Texture>::resize(self, size);
    }

    fn attachments(&self) -> Vec<AttachmentDescription> {
        vec![AttachmentDescription::new(self)]
    }
}

impl CanvasColorLayout for () {
    fn resize(&mut self, size: vek::Extent2<u16>) {}

    fn attachments(&self) -> Vec<AttachmentDescription> {
        Vec::new()
    }
}

macro_rules! tuple_impls_color_layout {
    ( $( $name:ident )+, $max:tt) => {
        impl<$($name: ColorTexel),+> CanvasColorLayout for ($(Texture2D<$name>,)+) {
            fn resize(&mut self, size: vek::Extent2<u16>) {
                seq!(N in 0..$max {
                    Texture2D::resize(&mut self.N, size);
                });
            }

            fn attachments(&self) -> Vec<AttachmentDescription> {
                Vec::new()
            }
        }
    };
}

tuple_impls_color_layout! { C0 C1, 2 }
tuple_impls_color_layout! { C0 C1 C2, 3 }
tuple_impls_color_layout! { C0 C1 C2 C3, 4 }
tuple_impls_color_layout! { C0 C1 C2 C3 C4, 5 }

impl<S: StencilOrDepthTexel> CanvasSpecialLayout for Texture2D<S> {
    fn resize(&mut self, size: vek::Extent2<u16>) {
        <Self as Texture>::resize(self, size);
    }
}

impl<D: DepthTexel, S: StencilTexel> CanvasSpecialLayout for (Texture2D<D>, Texture2D<S>) {
    fn resize(&mut self, size: vek::Extent2<u16>) {
        self.0.resize(size);
        self.1.resize(size);
    }
}

impl CanvasSpecialLayout for () {
    fn resize(&mut self, size: vek::Extent2<u16>) {}
}

impl<C: CanvasColorLayout, S: CanvasSpecialLayout> CanvasLayout for (C, S) {
    fn resize(&mut self, size: vek::Extent2<u16>) {
        self.0.resize(size);
        self.1.resize(size);
    }

    fn attachments() -> &'static [crate::prelude::TexelFormat] {
        todo!()
    }
}

impl CanvasLayout for () {
    fn resize(&mut self, size: vek::Extent2<u16>) {}

    fn attachments() -> &'static [crate::prelude::TexelFormat] {
        &[]
    }
}