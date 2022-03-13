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
    ClampToEdge(Option<vek::Vec4<f32>>),
    ClampToBorder(Option<vek::Vec4<f32>>),
    Repeat,
    MirroredRepeat,
}

// Texture dimensions
#[derive(EnumAsInner, Debug, Clone, Copy)]
pub enum TextureDimensions {
    Texture1d(u16),
    Texture2d(vek::Vec2<u16>),
    Texture3d(vek::Vec3<u16>),
    Texture2dArray(vek::Vec3<u16>),
}
