use crate::loader::Assets;
use std::{convert::Infallible, path::Path, sync::Arc};

/// File data what will be given to assets whenever we try to deserialize them.
///
/// We assume that all assets are files, even though they might not be.
///
/// Internally contains a [Asset Loader](crate::Assets) to allow for recursive asset loading.
pub struct Data<'a> {
    pub(super) name: &'a str,
    pub(super) extension: &'a str,
    pub(super) bytes: Arc<[u8]>,
    pub(super) path: &'a Path,
    pub(crate) loader: Option<&'a Assets>,
}

impl<'a> Data<'a> {
    /// Create a new "Data" struct that potentially contains a loader.
    pub fn new(
        name: &'a str,
        extension: &'a str,
        bytes: Arc<[u8]>,
        path: &'a Path,
        loader: Option<&'a Assets>,
    ) -> Self {
        Self {
            name,
            extension,
            bytes,
            path,
            loader,
        }
    }

    /// Get the name of the loaded file.
    pub fn name(&self) -> &str {
        self.name
    }

    /// Get the extension of the loaded file.
    pub fn extension(&self) -> &str {
        self.extension
    }

    /// Get the full path of the loaded file.
    pub fn path(&self) -> &Path {
        self.path
    }

    /// Get the bytes of the loaded file.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Get the recursive asset loader
    ///
    /// This is only Some when the user loads in a synchronous asset
    ///
    /// Can't use the recursive asset loader for async assets because it would cause deadlocks
    pub fn loader(&self) -> Option<&'a Assets> {
        self.loader
    }
}

/// An asset that will be loaded from a single unique file
///
/// Each asset can fail to load it's data
///
/// This trait contains a "context" that can be passed around with the asset load settings
pub trait Asset: Sized + 'static {
    /// Context that will be used to load the asset
    type Context<'ctx>;

    /// Settings that will be used to load the asset
    type Settings<'stg>;

    /// Possible error that we might return
    type Err: std::error::Error + Send + Sync + 'static;

    /// Possible extensions that are supported
    /// If this is of 0 length, then all extensions are supported
    fn extensions() -> &'static [&'static str];

    /// Deserialize the asset with the context and settings
    fn deserialize(
        data: Data,
        context: Self::Context<'_>,
        settings: Self::Settings<'_>,
    ) -> Result<Self, Self::Err>;
}

/// Just for convience's sake
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

// UTF8 string decoder
impl Asset for String {
    type Context<'ctx> = ();
    type Settings<'stg> = ();
    type Err = std::string::FromUtf8Error;

    fn extensions() -> &'static [&'static str] {
        &["txt"]
    }

    fn deserialize(
        data: Data,
        _: Self::Context<'_>,
        _: Self::Settings<'_>,
    ) -> Result<Self, Self::Err> {
        String::from_utf8(data.bytes().to_vec())
    }
}

// Raw binary reader
impl Asset for Vec<u8> {
    type Context<'ctx> = ();
    type Settings<'stg> = ();
    type Err = Infallible;

    fn extensions() -> &'static [&'static str] {
        &["bin"]
    }

    fn deserialize(
        data: Data,
        _context: Self::Context<'_>,
        _settings: Self::Settings<'_>,
    ) -> Result<Self, Self::Err> {
        Ok(data.bytes.to_vec())
    }
}
