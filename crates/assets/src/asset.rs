use crate::loader::CachedSlice;

// An asset that will be loaded from a single unique file
// Each asset has some extra data that can be used to construct the object
pub trait Asset<'args>: Sized {
    // Extra data that can be used to construct the object
    type Args: 'args;

    // The extensions supported by this asset
    fn extensions() -> &'static [&'static str];

    // Deserialize asset bytes, assuming that the given bytes are already in the valid format to deserialize
    fn deserialize(bytes: CachedSlice, args: Self::Args) -> Self;
}

impl Asset<'static> for String {
    type Args = ();

    fn extensions() -> &'static [&'static str] {
        &["txt"]
    }

    fn deserialize(bytes: CachedSlice, args: Self::Args) -> Self {
        String::from_utf8(bytes.as_ref().to_vec()).unwrap()
    }
}
