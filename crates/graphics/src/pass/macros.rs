use crate::pass::{
    ColorAttachments, ColorLayout, ColorOperations, LoadOp, Operation,
    RenderTarget, StoreOp,
};
use crate::prelude::BlendState;
use crate::format::{ColorTexel, Texel, TexelInfo};

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

        impl<$($name: ColorTexel),+> ColorOperations<($($name,)+)> for ($(Operation<$name>,)+) {
            fn operations(&self) -> Vec<wgpu::Operations<wgpu::Color>> {
                let mut vec = Vec::<wgpu::Operations<wgpu::Color>>::with_capacity($max);

                seq!(N in 0..$max {
                    let op = &self.N;

                    let load = match op.load {
                        LoadOp::Load => wgpu::LoadOp::Load,
                        LoadOp::Clear(color) => wgpu::LoadOp::Clear(<C~N as ColorTexel>::try_into_color(color).unwrap()),
                    };

                    let store = op.store;
                    vec.push(wgpu::Operations { load, store });
                });

                vec
            }
        }

        impl<'a, $($name: ColorTexel),+> ColorAttachments<'a, ($($name,)+)> for ($(RenderTarget<'a, $name>,)+) {
            fn views(&self) -> Vec<&'a wgpu::TextureView> {
                let mut vec = Vec::<&'a wgpu::TextureView>::with_capacity($max);

                seq!(N in 0..$max {
                    let val = &self.N;
                    vec.push(val.view());
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
