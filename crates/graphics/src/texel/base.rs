use bytemuck::Pod;

// Base numbers that are used to store the inner raw values of texture texels
pub trait Base: Pod + Clone + Send + Sync {
    const TYPE: BaseType; 
}

impl Base for i8 { const TYPE: BaseType = BaseType::SignedInt; }
impl Base for u8 { const TYPE: BaseType = BaseType::UnsignedInt; }
impl Base for i16 { const TYPE: BaseType = BaseType::SignedInt; }
impl Base for u16 { const TYPE: BaseType = BaseType::UnsignedInt; }
impl Base for i32 { const TYPE: BaseType = BaseType::SignedInt; }
impl Base for u32 { const TYPE: BaseType = BaseType::UnsignedInt; }
impl Base for i64 { const TYPE: BaseType = BaseType::SignedInt; }
impl Base for u64 { const TYPE: BaseType = BaseType::UnsignedInt; }
impl Base for half::f16 { const TYPE: BaseType = BaseType::Float; }
impl Base for f32 { const TYPE: BaseType = BaseType::Float; }
impl Base for f64 { const TYPE: BaseType = BaseType::Float; }

// Untyped representation needed for texel
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum BaseType {
    UnsignedInt,
    SignedInt,
    Float
}