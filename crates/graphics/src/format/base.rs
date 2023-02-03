use half::f16;

use crate::GpuPod;

// Base numbers that are used to store the inner raw values of texture texels
pub trait Base:
    Clone + Copy + Send + Sync + 'static + GpuPod
{
    const TYPE: BaseType;
    const SIGNED: bool;
}

macro_rules! impl_base {
    ($t:ty, $b: ident, $signed: ident) => {
        impl Base for $t {
            const TYPE: BaseType = BaseType::$b;
            const SIGNED: bool = $signed;
        }
    };
}

// Integer types
impl_base!(i8, Eight, true);
impl_base!(u8, Eight, false);
impl_base!(i16, Sixteen, true);
impl_base!(u16, Sixteen, false);
impl_base!(i32, ThirtyTwo, true);
impl_base!(u32, ThirtyTwo, false);

// Floating point types
impl_base!(f16, FloatSixteen, true);
impl_base!(f32, FloatThirtyTwo, true);

// Untyped representation of "base" needed for texel
// TODO: RENAME
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum BaseType {
    Eight,
    Sixteen,
    ThirtyTwo,
    FloatSixteen,
    FloatThirtyTwo,
}
