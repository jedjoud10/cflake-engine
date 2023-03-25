// Specialization constant trait for ease of use
// Does not implement GpuPod cause we need bools and spirq handles it automatically for us
pub trait SpecConstant {
    // Convert self to a spirq constant
    fn into_const_value(self) -> spirq::ConstantValue;
}

macro_rules! impl_spec_constant {
    ($t:ty) => {
        impl SpecConstant for $t {
            fn into_const_value(self) -> spirq::ConstantValue {
                spirq::ConstantValue::from(self)
            }
        }
    };
}

impl_spec_constant!(i32);
impl_spec_constant!(u32);
impl_spec_constant!(i64);
impl_spec_constant!(u64);
impl_spec_constant!(f32);
impl_spec_constant!(f64);
impl_spec_constant!(bool);
impl_spec_constant!([u8; 8]);
impl_spec_constant!([u8; 4]);
