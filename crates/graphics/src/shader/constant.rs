use paste::paste;

// Specialization constant enum
// Stored within an enum so we can differentiate between the types of values
// TODO: Implement 64 bit types
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SpecConstant {
    // i32 value as specialization constant
    I32(i32),

    // u32 value as specialization constant
    U32(u32),

    // f32 value as specialization constant
    F32(f32),

    // boolean value as specialization constant
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
