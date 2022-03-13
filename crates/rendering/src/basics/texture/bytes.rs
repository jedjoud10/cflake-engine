use enum_as_inner::EnumAsInner;

// Texture bytes
#[derive(EnumAsInner)]
pub enum TextureBytes {
    Loaded(Vec<u8>),
    Unloaded,
}