use std::path::{PathBuf, Path};

// File data is what will be given to assets whenever we try to deserialize them
// We will assume that all assets are files
pub struct Data<'a> {
    pub(super) name: &'a str,
    pub(super) extension: &'a str,
    pub(super) bytes: &'a [u8],
    pub(super) path: &'a Path,
}

impl<'a> Data<'a> {
    // Get the name of the loaded file
    pub fn name(&self) ->&str {
        &self.name
    }
    
    // Get the extension of the loaded file
    pub fn extension(&self) -> &str {
        &self.extension
    }

    // Get the full path of the loaded file
    pub fn path(&self) -> &Path {
        &self.path
    }
    
    // Get the bytes of the loaded file
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

// An asset that will be loaded from a single unique file
// Each asset has some extra data that can be used to construct the object
pub trait Asset<'args>: Sized {
    // Extra data that can be used to construct the object
    type Args: 'args;

    // The extensions supported by this asset
    fn extensions() -> &'static [&'static str];

    // Deserialize asset bytes, assuming that the given bytes are already in the valid format to deserialize
    fn deserialize(data: Data, args: Self::Args) -> Self;
}

impl Asset<'static> for String {
    type Args = ();

    fn extensions() -> &'static [&'static str] {
        &["txt"]
    }

    fn deserialize(data: Data, _args: Self::Args) -> Self {
        String::from_utf8(data.bytes().to_vec()).unwrap()
    }
}
