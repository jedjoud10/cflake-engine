use crate::{Asset, AssetInput, AssetLoadError, AsyncAsset};
use ahash::AHashMap;
use parking_lot::{Mutex, RwLock};

use utils::ThreadPool;

use std::{
    any::Any,
    ffi::OsStr,
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
};

// This is a handle to a specific asset that we are currently loading in
pub struct AsyncHandle<A: Asset> {
    _phantom: PhantomData<A>,
    index: usize,
}

// Used for async asset loading
type AsyncBoxedResult =
    Result<Box<dyn Any + Send + Sync>, AssetLoadError>;
type AsyncLoadedAssets = Mutex<Vec<Option<AsyncBoxedResult>>>;
type AsyncChannelResult = (AsyncBoxedResult, usize);
type AsyncLoadedBytes = Arc<RwLock<AHashMap<PathBuf, Arc<[u8]>>>>;

// Dynamic Asset Path specified by the user
type UserPath = Option<Arc<Path>>;

// Paths we can use to hijack default engine assets
// TODO: This might not be safe but tbh I couldn't care
type HijackPaths = AHashMap<PathBuf, PathBuf>;
type AsyncHijackPaths = Arc<RwLock<HijackPaths>>;

// This is the main asset manager resource that will load & cache newly loaded assets
// This asset manager will also contain the persistent assets that are included by default into the engine executable
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

    // Path that references the main user assets
    user: UserPath,
}

impl Assets {
    // Create a new asset loader using a path to the user defined asset folder (if there is one)
    pub fn new(user: Option<PathBuf>) -> Self {
        let user = user.map(|p| p.into());
        let (sender, receiver) =
            std::sync::mpsc::channel::<AsyncChannelResult>();

        Self {
            loaded: Default::default(),
            bytes: Default::default(),
            receiver,
            sender,
            user,
            hijack: Default::default(),
        }
    }

    // Import a persistent asset using it's global asset path and it's raw bytes
    pub fn import(&self, path: impl AsRef<Path>, bytes: Vec<u8>) {
        let path = path
            .as_ref()
            .strip_prefix("./assets/")
            .unwrap()
            .to_path_buf();
        self.bytes
            .write()
            .entry(path)
            .or_insert_with(|| Arc::from(bytes));
    }

    // Add a "hijack" path that will overwrite the path for a specific asset
    // This allows the user to write their own assets for engine assets if they want to
    pub fn hijack(
        &self,
        og: impl AsRef<Path>,
        new: impl AsRef<Path>,
    ) {
        let mut write = self.hijack.write();
        let og = og.as_ref().to_path_buf();
        let new = new.as_ref().to_path_buf();
        write.insert(og, new);
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
        ((A::extensions().contains(&extension))
            || A::extensions().is_empty())
        .then_some(())
        .ok_or_else(|| {
            AssetLoadError::InvalidExtension(extension.to_owned())
        })
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

    // Check if we must load the bytes dynamically or load cached bytes
    fn should_load_dynamically(
        bytes: &AsyncLoadedBytes,
        user: &UserPath,
        path: &Path,
    ) -> bool {
        if path.is_absolute() {
            true
        } else if user.is_some() {
            !bytes.read().contains_key(path)
        } else {
            false
        }
    }

    // Load bytes either dynamically or load cached bytes
    fn load_bytes(
        bytes: &AsyncLoadedBytes,
        hijack: AsyncHijackPaths,
        user: &UserPath,
        owned: PathBuf,
    ) -> Result<Arc<[u8]>, AssetLoadError> {
        // Check if we must load dynamically
        let dynamic =
            Self::should_load_dynamically(bytes, user, &owned);

        // Load the bytes dynamically or load them from cache
        if dynamic {
            Self::load_bytes_dynamically(bytes, hijack, user, owned)
        } else {
            Self::load_cached_bytes(bytes, &owned)
        }
    }

    // Load the already cached bytes
    fn load_cached_bytes(
        bytes: &AsyncLoadedBytes,
        path: &Path,
    ) -> Result<Arc<[u8]>, AssetLoadError> {
        log::debug!(
            "Loaded asset from path {:?} from cached bytes",
            path
        );

        bytes.read().get(path).cloned().ok_or_else(|| {
            let path = path.as_os_str().to_str().unwrap().to_owned();
            AssetLoadError::CachedNotFound(path)
        })
    }

    // Load the bytes for an asset dynamically and store them within self
    fn load_bytes_dynamically(
        bytes: &AsyncLoadedBytes,
        hijack: AsyncHijackPaths,
        user: &UserPath,
        owned: PathBuf,
    ) -> Result<Arc<[u8]>, AssetLoadError> {
        log::warn!(
            "Loading asset bytes from path {:?} dynamically...",
            &owned
        );
        let mut write = bytes.write();

        // Translate the path if it's defined
        let read = hijack.read();
        let owned = read.get(&owned).unwrap_or(&owned);

        // Get the path of the asset (make it absolute if needed)
        let path = if owned.is_absolute() {
            owned.clone()
        } else {
            // Sometimes the user path is not specified
            let user = user
                .as_ref()
                .ok_or(AssetLoadError::UserPathNotSpecified)?;

            let mut path = user.to_path_buf();
            path.push(owned.clone());
            path
        };

        // Load the asset dynamically
        let bytes = super::raw::read(&path)?;

        // Add the asset bytes into the cache
        let arc: Arc<[u8]> = Arc::from(bytes);
        write.insert(owned.clone(), arc.clone());
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
        user: UserPath,
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
            let bytes = Self::load_bytes(
                &bytes,
                hijack,
                &user,
                owned.clone(),
            )?;

            // Split the path into it's name and extension
            let (name, extension) = Self::decompose_path(&owned);

            // Deserialize the asset
            let asset = A::deserialize(
                crate::Data {
                    name,
                    extension,
                    bytes,
                    path: owned.as_path(),
                },
                context,
                settings,
            )
            .map_err(|err| {
                AssetLoadError::BoxedDeserialization(Box::new(err))
            })?;

            // Box the asset
            let boxed: Box<dyn Any + Send + Sync + 'static> =
                Box::new(asset);
            Ok(boxed)
        };

        // Send the result to the main thread
        sender.send((result(), index)).unwrap();
    }
}

// Synchronous loading
impl Assets {
    // Load an asset using some explicit/default loading arguments
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
        let bytes =
            Self::load_bytes(&self.bytes, hijack, &self.user, owned)?;

        // Deserialize the asset file
        A::deserialize(
            crate::Data {
                name,
                extension,
                bytes,
                path,
            },
            context,
            settings,
        )
        .map_err(|err| {
            AssetLoadError::BoxedDeserialization(Box::new(err))
        })
    }

    // Load multiple assets using some explicit/default loading arguments
    pub fn load_from_iter<'str, 'ctx, 'stg, A: Asset>(
        &self,
        inputs: impl IntoIterator<
            Item = impl AssetInput<'str, 'ctx, 'stg, A>,
        >,
    ) -> Vec<Result<A, AssetLoadError>> {
        inputs
            .into_iter()
            .map(|input| self.load(input))
            .collect::<Vec<Result<A, AssetLoadError>>>()
    }
}

// Asynchronous loading
impl Assets {
    // Load an asset using some explicit/default loading arguments in another thread
    pub fn async_load<'str, A: AsyncAsset>(
        &self,
        input: impl AssetInput<'str, 'static, 'static, A>,
        threadpool: &mut ThreadPool,
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
        let user = self.user.clone();
        let hijack = self.hijack.clone();

        // Create the handle's key
        let index = self.loaded.lock().len();
        let handle = AsyncHandle::<A> {
            _phantom: PhantomData,
            index,
        };
        self.loaded.lock().push(None);

        // Create a new task that will load this asset
        threadpool.execute(move || {
            Self::async_load_inner::<A>(
                owned, bytes, user, hijack, context, settings,
                sender, index,
            );
        });
        handle
    }

    // Load multiple assets using some explicit/default loading arguments in another thread
    // This returns handle(s) that we can wait for and fetch later on
    pub fn async_load_from_iter<'s, A: AsyncAsset>(
        &self,
        inputs: impl IntoIterator<
            Item = impl AssetInput<'s, 'static, 'static, A> + Send,
        >,
        threadpool: &mut ThreadPool,
    ) -> Vec<AsyncHandle<A>>
    where
        A::Settings<'static>: Send + Sync,
        A::Context<'static>: Send + Sync,
    {
        // Create a temporary threadpool scope for these assets only
        let mut outer = Vec::<AsyncHandle<A>>::new();
        let reference = &mut outer;
        let mut loaded = self.loaded.lock();
        threadpool.scope(move |scope| {
            for input in inputs.into_iter() {
                // Check the extension on a per file basis
                let (path, settings, context) = input.split();
                let path = Path::new(OsStr::new(path));
                log::debug!("Asynchronously loading asset {path:?} in batch...",);
                let owned = path.to_owned();

                // Clone the things that must be sent to the thread
                let bytes = self.bytes.clone();
                let sender = self.sender.clone();
                let user = self.user.clone();
                let hijack = self.hijack.clone();

                // Create the handle's key and insert it
                let index = loaded.len();
                reference.push(AsyncHandle::<A> {
                    _phantom: PhantomData,
                    index,
                });
                loaded.push(None);

                // Start telling worker threads to begin loading the assets
                scope.execute(move || {
                    Self::async_load_inner::<A>(
                        owned, bytes, user, hijack, context,
                        settings, sender, index,
                    );
                });
            }
        });
        outer
    }

    // Fetches the loaded assets from the receiver and caches them locally
    pub fn refresh(&mut self) {
        let loaded = self.loaded.get_mut();
        for (result, index) in self.receiver.try_iter() {
            let len = loaded.len().max(index + 1);
            loaded.resize_with(len, || None);

            loaded[index] = Some(result);
        }
    }

    // This will check if the asset loader finished loading a specific asset using it's handle
    pub fn has_finished_loading<A: AsyncAsset>(
        &mut self,
        handle: &AsyncHandle<A>,
    ) -> bool {
        self.refresh();
        self.loaded
            .get_mut()
            .get(handle.index)
            .map(|x| x.is_some())
            .unwrap_or_default()
    }

    // This will wait until the asset referenced by this handle has finished loading
    pub fn wait<A: AsyncAsset>(
        &mut self,
        handle: AsyncHandle<A>,
    ) -> Result<A, AssetLoadError> {
        // Spin lock whilst whilst waiting for an asset to load
        while !self.has_finished_loading(&handle) {
            std::hint::spin_loop();
        }

        // Replace the slot with None
        let loaded = self.loaded.get_mut();
        let old = loaded[handle.index].take().unwrap();
        old.map(|b| *b.downcast::<A>().unwrap())
    }

    // This will wait until all the assets reference by these handles have finished loading
    pub fn wait_from_iter<A: AsyncAsset>(
        &mut self,
        handles: impl IntoIterator<Item = AsyncHandle<A>>,
    ) -> Vec<Result<A, AssetLoadError>> {
        log::debug!("Waiting for async assets to load...");
        handles
            .into_iter()
            .map(|handle| self.wait(handle))
            .collect::<Vec<_>>()
    }
}
