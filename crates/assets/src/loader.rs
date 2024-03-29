use crate::{Asset, AssetInput, AssetLoadError, AsyncAsset};
use ahash::AHashMap;
use parking_lot::{Mutex, RwLock};
use std::{
    any::Any,
    ffi::OsStr,
    marker::PhantomData,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
};

/// This is a handle to a specific asset that we are currently loading in asynchronously
///
/// This can be used to fetch for the state of the asset and to fetch for it after it loaded completely
pub struct AsyncHandle<A: Asset> {
    _phantom: PhantomData<A>,
    index: usize,
}

// Used for async asset loading
type AsyncBoxedResult = Result<Box<dyn Any + Send + Sync>, AssetLoadError>;
type AsyncLoadedAssets = Mutex<Vec<Option<AsyncBoxedResult>>>;
type AsyncChannelResult = (AsyncBoxedResult, usize);
type AsyncLoadedBytes = Arc<RwLock<AHashMap<PathBuf, Arc<[u8]>>>>;

// Paths we can use to hijack default engine assets
// TODO: This might not be safe but tbh I couldn't care
type HijackPaths = AHashMap<PathBuf, PathBuf>;
type AsyncHijackPaths = Arc<RwLock<HijackPaths>>;

pub use cfg_if;
pub use include_dir;
pub use with_builtin_macros;

/// This is the main asset manager resource that will load & cache newly loaded assets
///
/// This asset manager will also contain the persistent assets that are included by default into the engine executable
pub struct Assets {
    // Receiver that will keep track of the assets that were loaded
    sender: Sender<AsyncChannelResult>,
    receiver: Receiver<AsyncChannelResult>,

    // Keep track of the assets that were sucessfully loaded
    // The value corresponding to each key might be None in the case that the asset did not load (yet)
    loaded: AsyncLoadedAssets,

    // We can use these re-definitions to allow the user to change default asset paths
    hijack: AsyncHijackPaths,

    // Keep track of the bytes that were loaded in other threads
    // The value might be none in the case that the bytes were not loaded
    // The path buf contains the local path of each asset
    bytes: AsyncLoadedBytes,
}

impl Default for Assets {
    fn default() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel::<AsyncChannelResult>();

        Self {
            loaded: Default::default(),
            bytes: Default::default(),
            receiver,
            sender,
            hijack: Default::default(),
        }
    }
}

impl Assets {
    /// Create a new asset loader using a pre-defined user assets (if supplied).
    pub fn new() -> Self {
        Self::default()
    }

    /// Import a persistent asset using it's asset path (not global) and it's raw bytes.
    pub fn import(&self, path: impl AsRef<Path>, bytes: Vec<u8>) {
        assert!(!path.as_ref().is_absolute(), "Told you, dumbass");

        // Only strip the prefix if needed
        let path = path
            .as_ref()
            .strip_prefix("./assets/")
            .unwrap_or(path.as_ref())
            .to_path_buf();

        self.bytes
            .write()
            .entry(path)
            .or_insert_with(|| Arc::from(bytes));
    }

    /// Add a "hijack" path that will overwrite the path for a specific asset.
    /// This allows users to overwrite engine assets using their own custom assets.
    pub fn hijack(&self, og: impl AsRef<Path>, new: impl AsRef<Path>) {
        let mut write = self.hijack.write();
        let og = og.as_ref().to_path_buf();
        let new = new.as_ref().to_path_buf();
        write.insert(og, new);
    }

    /// Uncache the bytes of an already cached asset. Used for hot-reloading.
    pub fn uncache(&self, path: &str) -> Option<()> {
        let path = Path::new(path);
        let mut bytes = self.bytes.write();
        bytes.remove(path)?;
        log::debug!("Un-cached the bytes of asset {:?}", path);
        Some(())
    }

    /// Checks if the asset loader will load in assets at runtime or if they will be packed.
    pub fn packed(&self) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(feature = "pack-assets")] {
                true
            } else {
                false
            }
        }
    }

    /// Get the global path of the file that is used by an asset.
    pub fn path(&self, asset: &str) -> Option<PathBuf> {
        let owned = PathBuf::from_str(asset).ok()?;
        let read = self.hijack.read();
        let owned = read.get(&owned).unwrap_or(&owned);
        owned.is_absolute().then(|| owned.clone())
    }
}

// Helper functions
impl Assets {
    // Check if the extension of a file is valid
    fn validate<A: Asset>(path: &Path) -> Result<(), AssetLoadError> {
        let (_, extension) = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or(AssetLoadError::InvalidOsStr)?
            .split_once('.')
            .ok_or(AssetLoadError::MissingExtension)?;

        // If the asset has no extensions, we shall not check
        ((A::extensions().contains(&extension)) || A::extensions().is_empty())
            .then_some(())
            .ok_or_else(|| AssetLoadError::InvalidExtension(extension.to_owned()))
    }

    // Convert a path to it's raw name and extension
    fn decompose_path(path: &Path) -> (&str, &str) {
        let (name, extension) = path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap()
            .split_once('.')
            .unwrap();
        (name, extension)
    }

    // Load bytes either dynamically or load cached bytes
    fn load_bytes(
        bytes: &AsyncLoadedBytes,
        hijack: AsyncHijackPaths,
        owned: PathBuf,
    ) -> Result<Arc<[u8]>, AssetLoadError> {
        // Load the bytes from cached bytes first
        let mut loaded = Self::load_cached_bytes(bytes, &owned);

        // If that fails, try loading from user defined asset path
        if let Err(AssetLoadError::CachedNotFound(_)) = &loaded {
            loaded = Self::load_bytes_dynamically(bytes, hijack, owned);
        }

        // Return the (hopefully loaded) asset
        loaded
    }

    // Load the already cached bytes
    fn load_cached_bytes(
        bytes: &AsyncLoadedBytes,
        path: &Path,
    ) -> Result<Arc<[u8]>, AssetLoadError> {
        if let Some(bytes) = bytes.read().get(path).cloned() {
            log::debug!("Loaded asset from path {:?} from cached bytes", path);
            Ok(bytes.clone())
        } else {
            let path = path.as_os_str().to_str().unwrap().to_owned();
            Err(AssetLoadError::CachedNotFound(path))
        }
    }

    // Load the bytes for an asset dynamically and store them within self
    fn load_bytes_dynamically(
        bytes: &AsyncLoadedBytes,
        hijack: AsyncHijackPaths,
        owned: PathBuf,
    ) -> Result<Arc<[u8]>, AssetLoadError> {
        let og = owned.clone();
        log::warn!("Loading asset bytes from path {:?} dynamically...", &owned);
        let mut write = bytes.write();

        // Translate the path if it's defined
        let read = hijack.read();
        let owned = read.get(&owned).unwrap_or(&owned);

        // Get the path of the asset
        let path = if owned.is_absolute() {
            owned.clone()
        } else {
            let path = owned.as_os_str().to_str().unwrap().to_owned();
            return Err(AssetLoadError::CachedNotFound(path));
        };

        // Load the asset dynamically
        let bytes = super::raw::read(&path)?;

        // Add the asset bytes into the cache
        let arc: Arc<[u8]> = Arc::from(bytes);
        write.insert(og, arc.clone());
        log::debug!(
            "Successfully loaded dynamic asset bytes from path {:?}",
            &owned
        );
        Ok(arc)
    }

    // Load an asset asynchronously and automatically add it to the loaded assets
    fn async_load_inner<A: AsyncAsset>(
        owned: PathBuf,
        bytes: AsyncLoadedBytes,
        hijack: AsyncHijackPaths,
        context: <A as Asset>::Context<'_>,
        settings: <A as Asset>::Settings<'_>,
        sender: Sender<AsyncChannelResult>,
        index: usize,
    ) {
        // Smaller scope so we can use ? internally
        let result = move || {
            // Validate the path and extensions
            Self::validate::<A>(&owned)?;

            // Load the bytes dynamically or from cache
            let bytes = Self::load_bytes(&bytes, hijack, owned.clone())?;

            // Split the path into it's name and extension
            let (name, extension) = Self::decompose_path(&owned);

            // Deserialize the asset
            let asset = A::deserialize(
                crate::Data {
                    name,
                    extension,
                    bytes,
                    path: owned.as_path(),
                    loader: None,
                },
                context,
                settings,
            )
            .map_err(|err| AssetLoadError::BoxedDeserialization(Box::new(err)))?;

            // Box the asset
            let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(asset);
            Ok(boxed)
        };

        // Send the result to the main thread
        sender.send((result(), index)).unwrap();
    }
}

// Synchronous loading
impl Assets {
    /// Load an asset using some an implementation of an [Asset Input](crate::AssetInput)
    pub fn load<'str, 'ctx, 'stg, A: Asset>(
        &self,
        input: impl AssetInput<'str, 'ctx, 'stg, A>,
    ) -> Result<A, AssetLoadError> {
        // Check if the extension is valid
        let (path, settings, context) = input.split();
        let path = Path::new(OsStr::new(path));
        let owned = path.to_owned();
        Self::validate::<A>(path)?;
        log::debug!("Synchronously loading asset {path:?}...",);

        // All this does is that it ensures that the bytes are valid before we actually deserialize the asset
        let (name, extension) = Self::decompose_path(path);

        // Load the asset bytes (either dynamically or fetch cached bytes)
        let hijack = self.hijack.clone();
        let bytes = Self::load_bytes(&self.bytes, hijack, owned)?;

        // Deserialize the asset file
        A::deserialize(
            crate::Data {
                name,
                extension,
                bytes,
                path,
                loader: Some(self),
            },
            context,
            settings,
        )
        .map_err(|err| AssetLoadError::BoxedDeserialization(Box::new(err)))
    }

    /// Load multiple assets using a common implementation of an [Asset Input](crate::AssetInput)
    pub fn load_from_iter<'str, 'ctx, 'stg, A: Asset>(
        &self,
        inputs: impl IntoIterator<Item = impl AssetInput<'str, 'ctx, 'stg, A>>,
    ) -> Vec<Result<A, AssetLoadError>> {
        inputs
            .into_iter()
            .map(|input| self.load(input))
            .collect::<Vec<Result<A, AssetLoadError>>>()
    }
}

// Asynchronous loading
impl Assets {
    /// Load an asset using some an implementation of an [Asset Input](crate::AssetInput) in another thread.
    /// Thread management is handled by [rayon].
    pub fn async_load<'str, A: AsyncAsset>(
        &self,
        input: impl AssetInput<'str, 'static, 'static, A>,
    ) -> AsyncHandle<A>
    where
        A::Settings<'static>: Send + Sync,
        A::Context<'static>: Send + Sync,
    {
        // Get the path and arguments
        let (path, settings, context) = input.split();
        let path = Path::new(OsStr::new(path));
        let owned = path.to_owned();
        log::debug!("Asynchronously loading asset {path:?}...",);

        // Clone the things that must be sent to the thread
        let bytes = self.bytes.clone();
        let sender = self.sender.clone();
        let hijack = self.hijack.clone();

        // Create the handle's key
        let index = self.loaded.lock().len();
        let handle = AsyncHandle::<A> {
            _phantom: PhantomData,
            index,
        };
        self.loaded.lock().push(None);

        // Create a new task that will load this asset
        rayon::spawn(move || {
            Self::async_load_inner::<A>(owned, bytes, hijack, context, settings, sender, index);
        });
        handle
    }

    /// Load multiple assets using a common implementation of an [Asset Input](crate::AssetInput) in another thread.
    /// Returns the [async handles](crate::AsyncHandle) that we can wait for and fetch later on.
    pub fn async_load_from_iter<'s, A: AsyncAsset>(
        &self,
        inputs: impl IntoIterator<Item = impl AssetInput<'s, 'static, 'static, A> + Send>,
    ) -> Vec<AsyncHandle<A>>
    where
        A::Settings<'static>: Send + Sync,
        A::Context<'static>: Send + Sync,
    {
        // Create a temporary threadpool scope for these assets only
        let mut outer = Vec::<AsyncHandle<A>>::new();
        let reference = &mut outer;
        let mut loaded = self.loaded.lock();

        for input in inputs.into_iter() {
            // Check the extension on a per file basis
            let (path, settings, context) = input.split();
            let path = Path::new(OsStr::new(path));
            log::debug!("Asynchronously loading asset {path:?} in batch...",);
            let owned = path.to_owned();

            // Clone the things that must be sent to the thread
            let bytes = self.bytes.clone();
            let sender = self.sender.clone();
            let hijack = self.hijack.clone();

            // Create the handle's key and insert it
            let index = loaded.len();
            reference.push(AsyncHandle::<A> {
                _phantom: PhantomData,
                index,
            });
            loaded.push(None);

            // Start telling worker threads to begin loading the assets
            rayon::spawn(move || {
                Self::async_load_inner::<A>(owned, bytes, hijack, context, settings, sender, index);
            });
        }
        outer
    }

    /// Fetches the loaded assets from the receiver and caches them locally.
    pub fn refresh(&self) {
        let mut loaded = self.loaded.lock();
        for (result, index) in self.receiver.try_iter() {
            let len = loaded.len().max(index + 1);
            loaded.resize_with(len, || None);
            loaded[index] = Some(result);
        }
    }

    /// This will check if the asset loader finished loading a specific asset using it's handle.
    pub fn has_finished_loading<A: AsyncAsset>(&self, handle: &AsyncHandle<A>) -> bool {
        self.refresh();
        self.loaded
            .lock()
            .get(handle.index)
            .map(|x| x.is_some())
            .unwrap_or_default()
    }

    /// This will wait until the asset referenced by this handle has finished loading.
    pub fn wait<A: AsyncAsset>(&self, handle: AsyncHandle<A>) -> Result<A, AssetLoadError> {
        // Spin lock whilst whilst waiting for an asset to load
        while !self.has_finished_loading(&handle) {
            std::hint::spin_loop();
        }

        // Replace the slot with None
        let mut loaded = self.loaded.lock();
        let old = loaded[handle.index].take().unwrap();
        old.map(|b| *b.downcast::<A>().unwrap())
    }

    /// This will wait until all the assets reference by these handles have finished loading.
    pub fn wait_from_iter<A: AsyncAsset>(
        &self,
        handles: impl IntoIterator<Item = AsyncHandle<A>>,
    ) -> Vec<Result<A, AssetLoadError>> {
        // TODO: Optimize this by not waiting for assets one by one
        log::debug!("Waiting for async assets to load...");
        handles
            .into_iter()
            .map(|handle| self.wait(handle))
            .collect::<Vec<_>>()
    }
}
