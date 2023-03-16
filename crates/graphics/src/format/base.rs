use crate::GpuPod;
use half::f16;
use paste::paste;
use crate::ElementType;

// Base underlying type used for TextureFormat and VertexFormat
pub trait Base:
    Clone + Copy + Send + Sync + 'static + GpuPod
{
    const ELEMENT_TYPE: ElementType;
}

macro_rules! impl_base {
    ($t:ty, $b: expr) => {
        impl Base for $t {
            const ELEMENT_TYPE: ElementType = $b;
        }
    };
}

// Integer types
impl_base!(i8, ElementType::Eight { signed: true, normalized: false });
impl_base!(u8, ElementType::Eight { signed: false, normalized: false } );
impl_base!(i16, ElementType::Sixteen { signed: true, normalized: false });
impl_base!(u16, ElementType::Sixteen { signed: false, normalized: false });
impl_base!(i32, ElementType::ThirtyTwo { signed: true });
impl_base!(u32, ElementType::ThirtyTwo { signed: false });

// Floating point types
impl_base!(f16, ElementType::FloatSixteen);
impl_base!(f32, ElementType::FloatThirtyTwo);