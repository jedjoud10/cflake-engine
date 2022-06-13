use std::marker::PhantomData;
use vek::{Vec2, Vec3, Vec4};

// Base numbers that are used to store the inner raw values of texture texels
pub trait Base {}
impl Base for i8 {}
impl Base for u8 {}
impl Base for i16 {}
impl Base for u16 {}
impl Base for i32 {}
impl Base for u32 {}
impl Base for f32 {}

// Elements are just values that can be stored within channels, like u32, Normalized<i8> or i8
pub trait Element {}
impl<T: Base> Element for T {}


// The channels that represent the texels
pub struct R<T: Element>(PhantomData<T>);
pub struct RG<T: Element>(PhantomData<Vec2<T>>);
pub struct RGB<T: Element>(PhantomData<Vec3<T>>);
pub struct RGBA<T: Element>(PhantomData<Vec4<T>>);

// Unique depth and stencil channels for depth render textures and stencil render textures
pub struct Depth<T: Element>(PhantomData<T>);
pub struct Stencil<T: Element>(PhantomData<T>);