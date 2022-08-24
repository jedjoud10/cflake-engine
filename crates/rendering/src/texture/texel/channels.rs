use super::Element;
use std::marker::PhantomData;
use vek::{Vec2, Vec3, Vec4};

// The channels that represent the texels
pub struct R<T: Element>(PhantomData<T>);
pub struct RG<T: Element>(PhantomData<Vec2<T>>);
pub struct RGB<T: Element>(PhantomData<Vec3<T>>);
pub struct RGBA<T: Element>(PhantomData<Vec4<T>>);

// Unique depth and stencil channels for depth render textures and stencil render textures
pub struct Depth<T: Element>(PhantomData<T>);
pub struct Stencil<T: Element>(PhantomData<T>);

// Gamma corrected RGB and RGBA channels
pub struct SRGB<T: Element>(PhantomData<T>);
pub struct SRGBA<T: Element>(PhantomData<T>);
