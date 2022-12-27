use bytemuck::Pod;
use half::f16;

// Base numbers that are used to store the inner raw values of texture texels
pub trait Base: Pod + Clone + Send + Sync {
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

impl_base!(i8, Eight, true);
impl_base!(u8, Eight, false);
impl_base!(i16, Sixteen, true);
impl_base!(u16, Sixteen, false);
impl_base!(i32, ThirtyTwo, true);
impl_base!(u32, ThirtyTwo, false);
impl_base!(i64, SixtyFour, true);
impl_base!(u64, SixtyFour, false);

impl_base!(f16, FloatSixteen, true);
impl_base!(f32, FloatThirtyTwo, true);
impl_base!(f64, FloatSixtyFour, true);

// Untyped representation of "base" needed for texel
// TODO: RENAME
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum BaseType {
    Eight, Sixteen, ThirtyTwo, SixtyFour,
    FloatSixteen, FloatThirtyTwo, FloatSixtyFour
}