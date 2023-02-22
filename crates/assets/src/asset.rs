use std::{path::Path, sync::Arc};

// File data is what will be given to assets whenever we try to deserialize them
// We will assume that all assets are files
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
// Each asset can fail to load it's data
// This trait contains a "context" that can be passed around with the asset load settings
pub trait Asset: Sized + 'static {
    type Context<'ctx>;
    type Settings<'stg>;

    // Possible error that we might return
    type Err: std::error::Error + Send + Sync + 'static;

    // Possible extensions that are supported
    // If this is of 0 length, then all extensions are supported
    fn extensions() -> &'static [&'static str];

    // Deserialize the asset with the context and settings
    fn deserialize<'c, 's>(
        data: Data,
        context: Self::Context<'c>,
        settings: Self::Settings<'s>,
    ) -> Result<Self, Self::Err>;
}

// Just for convience's sake
pub trait AsyncAsset: Asset + Send + Sync
where
    <Self as Asset>::Err: Send,
{
}

impl<T: Asset + Send + Sync> AsyncAsset for T
where
    T::Context<'static>: 'static + Send + Sync,
    T::Settings<'static>: 'static + Send + Sync,
    T::Err: 'static + Send + Sync,
{
}

impl Asset for String {
    type Context<'ctx> = ();
    type Settings<'stg> = ();
    type Err = std::string::FromUtf8Error;

    fn extensions() -> &'static [&'static str] {
        &["txt"]
    }

    fn deserialize<'c, 's>(
        data: Data,
        _: Self::Context<'c>,
        _: Self::Settings<'s>,
    ) -> Result<Self, Self::Err> {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        String::from_utf8(data.bytes().to_vec())
    }
}
