use paste::paste;

// Specialization constant enum
// Stored within an enum so we can differentiate between the types of values
// TODO: Implement 64 bit types
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SpecConstant {
    I32(i32),
    U32(u32),
    F32(f32),
    BOOL(bool),
}

macro_rules! impl_spec_constant {
    ($t:ty, $v:ty) => {
        paste! {
            impl Into<SpecConstant> for $t {
                fn into(self) -> SpecConstant {
                    $v(self)
                }
            }
        }
    };
}

impl_spec_constant!(i32, SpecConstant::I32);
impl_spec_constant!(u32, SpecConstant::U32);
impl_spec_constant!(f32, SpecConstant::F32);
impl_spec_constant!(bool, SpecConstant::BOOL);
