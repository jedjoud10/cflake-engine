use std::{path::Path, sync::Arc};

// File data is what will be given to assets whenever we try to deserialize them
// We will assume that all assets are files
// TODO: add the loader back again
pub struct Data<'a> {
    pub(super) name: &'a str,
    pub(super) extension: &'a str,
    pub(super) bytes: Arc<[u8]>,
    pub(super) path: &'a Path,
}

impl<'a> Data<'a> {
    // Get the name of the loaded file
    pub fn name(&self) -> &str {
        self.name
    }

    // Get the extension of the loaded file
    pub fn extension(&self) -> &str {
        self.extension
    }

    // Get the full path of the loaded file
    pub fn path(&self) -> &Path {
        self.path
    }

    // Get the bytes of the loaded file
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

// An asset that will be loaded from a single unique file
// Each asset has some extra data that can be used to construct the object
pub trait Asset: Sized + 'static {
    type Args<'args>;
    type Result;
    fn extensions() -> &'static [&'static str];
    fn deserialize<'args>(
        data: Data,
        args: Self::Args<'args>,
    ) -> Self::Result;
}

// Just for convience's sake
pub trait AsyncAsset: Asset + Sync + Send {}
impl<T: Asset + Send + Sync> AsyncAsset for T where
    T::Args<'static>: 'static + Send + Sync
{
}

impl Asset for String {
    type Args<'args> = ();
    type Result = Result<Self, std::string::FromUtf8Error>;

    fn extensions() -> &'static [&'static str] {
        &["txt"]
    }

    fn deserialize<'args>(
        data: Data,
        _args: Self::Args<'args>,
    ) -> Self::Result {
        std::thread::sleep(std::time::Duration::from_millis(1));
        String::from_utf8(data.bytes().to_vec())
    }
}
