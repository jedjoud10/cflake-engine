use enum_as_inner::EnumAsInner;

// Texture filters
#[derive(Debug, Clone, Copy)]
pub enum TextureFilter {
    Linear,
    Nearest,
}

// Texture wrapping filters
#[derive(Debug, Clone, Copy)]
pub enum TextureWrapMode {
    ClampToEdge(Option<veclib::Vector4<f32>>),
    ClampToBorder(Option<veclib::Vector4<f32>>),
    Repeat,
    MirroredRepeat,
}

// Texture dimensions
#[derive(EnumAsInner, Debug, Clone, Copy)]
pub enum TextureDimensions {
    Texture1d(u16),
    Texture2d(veclib::Vector2<u16>),
    Texture3d(veclib::Vector3<u16>),
    Texture2dArray(veclib::Vector3<u16>),
}
