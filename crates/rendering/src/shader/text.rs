use assets::Asset;

// This is some raw shader text that can be loaded from any shader file
pub struct RawText(pub(super) String);

impl Asset<'static> for RawText {
    type Args = ();

    fn extensions() -> &'static [&'static str] {
        &[".vrsh.glsl", ".frsh.glsl", "cmpt.glsl", ""]
    }

    fn deserialize(bytes: assets::loader::CachedSlice, args: Self::Args) -> Self {
        todo!()
    }
} 