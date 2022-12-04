use crate::{Asset, AssetInput, AsyncAsset};
use ahash::AHashMap;
use parking_lot::RwLock;
use slotmap::{DefaultKey, SlotMap};
use thiserror::Error;
use utils::ThreadPool;

use std::{
    any::Any,
    cell::RefCell,
    ffi::OsStr,
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    }, error::Error,
};

// This is a handle to a specific asset that we are currently loading in
pub struct AsyncHandle<A: Asset> {
    _phantom: PhantomData<A>,
    key: DefaultKey,
}

// Error that occurs when we try to load an asset
#[derive(Error, Debug)]
pub enum AssetLoadError {
    #[error("Invalid '{0}' extension in file path")]
    InvalidExtension(String),

    #[error("Cannot find file at path '{0}'")]
    DynamicNotFound(String),

    #[error("Could not convert to OS str")]
    InvalidOsStr,

    #[error("Missing extension in file path")]
    MissingExtension,

    #[error("User asset path was not specified")]
    UserPathNotSpecified,

    #[error("Deserialization error {0}")]
    BoxedDeserialization(Box<dyn Error + Send + Sync>),
}

// Used for async asset loading
type AsyncBoxedResult = Result<Box<dyn Any + Send + Sync>, AssetLoadError>;
type AsyncSlotMap = SlotMap<DefaultKey, Option<AsyncBoxedResult>>;
type AsyncLoadedAssets = Arc<RwLock<AsyncSlotMap>>;
type AsyncLoadedBytes = Arc<RwLock<AHashMap<PathBuf, Arc<[u8]>>>>;

// Dynamic Asset Path specified by the user 
type UserPath = Option<Arc<Path>>;

// This is the main asset manager resource that will load & cache newly loaded assets
// This asset manager will also contain the persistent assets that are included by default into the engine executable
pub struct Assets {
    // Keep track of the assets that were sucessfully loaded
    // The value corresponding to each key might be None in the case that the asset did not load (yet)
    assets: AsyncLoadedAssets,

    // Keep track of the bytes that were loaded in other threads
    // The value might be none in the case that the bytes were not loaded
    bytes: AsyncLoadedBytes,

    // This receiver and vec keep track of the key IDs of items that were loaded in
    receiver: Receiver<DefaultKey>,
    sender: Sender<DefaultKey>,
    loaded: RefCell<Vec<DefaultKey>>,

    // Path that references the main user assets
    user: UserPath,
}

impl Assets {
    // Create a new asset loader using a path to the user defined asset folder (if there is one)
    pub fn new(user: Option<PathBuf>) -> Self {
        let (sender, receiver) =
            std::sync::mpsc::channel::<DefaultKey>();

        let user = user.map(|p| p.into());

        Self {
            assets: Default::default(),
            bytes: Default::default(),
            loaded: Default::default(),
            receiver,
            sender,
            user,
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
}

// Helper functions
impl Assets {
    // Check if the extension of a file is valid
    fn validate<A: Asset>(path: &Path) -> Result<(), AssetLoadError> {
        let (_, extension) = path
            .file_name()
            .and_then(OsStr::to_str).ok_or(AssetLoadError::InvalidOsStr)?
            .split_once('.').ok_or(AssetLoadError::MissingExtension)?;

        // If the asset has no extensions, we shall not check
        ((A::extensions().contains(&extension))
            || A::extensions().is_empty())
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
    fn load_bytes(bytes: &AsyncLoadedBytes, user: &UserPath, owned: PathBuf) -> Result<Arc<[u8]>, AssetLoadError> {
        let bytes = if bytes.read().contains_key(&owned) {
            Self::load_cached_bytes(bytes, &owned)
        } else {
            Self::load_bytes_dynamically(bytes, user, owned)?
        };
        Ok(bytes)
    }

    // Load the already cached bytes
    fn load_cached_bytes(bytes: &AsyncLoadedBytes, path: &Path) -> Arc<[u8]> {
        log::debug!("Loaded asset from path {:?} from cached bytes", path);
        bytes.read().get(path).unwrap().clone()
    }

    // Load the bytes for an asset dynamically and store them within self
    fn load_bytes_dynamically(bytes: &AsyncLoadedBytes, user: &UserPath, owned: PathBuf) -> Result<Arc<[u8]>, AssetLoadError> {
        log::debug!("Loading asset bytes from path {:?} dynamically...", &owned);
        let mut write = bytes.write();
        let user = user.as_ref().ok_or(AssetLoadError::UserPathNotSpecified)?;
        let bytes = super::raw::read(&owned, user)?;
        let arc: Arc<[u8]> = Arc::from(bytes);
        write.insert(owned.clone(), arc.clone());
        log::debug!("Successfully loaded asset bytes from path {:?}", &owned);
        Ok(arc)
    }

    // Load an asset asynchronously and automatically add it to the loaded assets
    // TODO: Rewrite this function
    fn async_load_inner<A: AsyncAsset>(
        owned: PathBuf,
        bytes: AsyncLoadedBytes,
        user: UserPath,
        args: <A as Asset>::Args<'_>,
        assets: AsyncLoadedAssets,
        key: DefaultKey,
        sender: Sender<DefaultKey>
    ) {
        // Smaller scope so we can use ? internally
        let result = move || {
            // Validate the path and extensions
            Self::validate::<A>(&owned)?;

            // Load the bytes dynamically or from cache
            let bytes =  Self::load_bytes(&bytes, &user, owned.clone())?;

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
                args,
            ).map_err(|err| AssetLoadError::BoxedDeserialization(Box::new(err)))?;

            // Box the asset
            let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(asset);
            Ok(boxed)
        };

        // Send the result to the main thread
        let mut write = assets.write();
        let opt = write.get_mut(key).unwrap();
        *opt = Some(result());
        sender.send(key).unwrap();
    }
}

// Synchronous loading
impl Assets {
    // Load an asset using some explicit/default loading arguments
    pub fn load<'s, 'args, A: Asset>(
        &self,
        input: impl AssetInput<'s, 'args, A>,
    ) -> Result<A, AssetLoadError> {
        // Check if the extension is valid
        let (path, args) = input.split();
        let path = Path::new(OsStr::new(path));
        let owned = path.to_owned();
        Self::validate::<A>(&path)?;

        // All this does is that it ensures that the bytes are valid before we actually deserialize the asset
        let (name, extension) = Self::decompose_path(path);
        
        // Load the asset bytes (either dynamically or fetch cached bytes)
        let bytes = Self::load_bytes(&self.bytes, &self.user, owned)?;

        // Deserialize the asset file
        A::deserialize(
            crate::Data {
                name,
                extension,
                bytes,
                path: &path,
            },
            args,
        ).map_err(|err| AssetLoadError::BoxedDeserialization(Box::new(err)))
    }

    // Load multiple assets using some explicit/default loading arguments
    pub fn load_from_iter<'s, 'args, A: Asset>(
        &self,
        inputs: impl IntoIterator<Item = impl AssetInput<'s, 'args, A>>,
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
    pub fn async_load<'s, A: AsyncAsset>(
        &self,
        input: impl AssetInput<'s, 'static, A>,
        threadpool: &mut ThreadPool,
    ) -> AsyncHandle<A>
    where
        A::Args<'static>: Send + Sync,
    {
        // Get the path and arguments
        let (path, args) = input.split();
        let path = Path::new(OsStr::new(path));
        let owned = path.to_owned();

        // Clone the things that must be sent to the thread
        let assets = self.assets.clone();
        let bytes = self.bytes.clone();
        let sender = self.sender.clone();
        let user = self.user.clone();

        // Create the handle's key
        let key = self.assets.write().insert(None);
        let handle = AsyncHandle::<A> {
            _phantom: PhantomData,
            key,
        };

        // Create a new task that will load this asset
        threadpool.execute(move || {
            Self::async_load_inner::<A>(owned, bytes, user, args, assets, key, sender);
        });
        handle
    }

    // Load multiple assets using some explicit/default loading arguments in another thread
    // This returns handle(s) that we can wait for and fetch later on
    pub fn async_load_from_iter<'s, A: AsyncAsset>(
        &self,
        inputs: impl IntoIterator<
            Item = impl AssetInput<'s, 'static, A> + Send,
        >,
        threadpool: &mut ThreadPool,
    ) -> Vec<AsyncHandle<A>>
    where
        A::Args<'static>: Send + Sync
    {
        // Create a temporary threadpool scope for these assets only
        let mut outer = Vec::<AsyncHandle<A>>::new();
        let reference = &mut outer;
        threadpool.scope(move |scope| {
            for input in inputs.into_iter() {
                // Check the extension on a per file basis
                let (path, args) = input.split();
                let path = Path::new(OsStr::new(path));
                let owned = path.to_owned();

                // Clone the things that must be sent to the thread
                let assets = self.assets.clone();
                let bytes = self.bytes.clone();
                let sender = self.sender.clone();
                let user = self.user.clone();

                // Create the handle's key and insert it
                let key = self.assets.write().insert(None);
                reference.push(AsyncHandle::<A> {
                    _phantom: PhantomData,
                    key,
                });

                scope.execute(move || {
                    Self::async_load_inner::<A>(owned, bytes, user, args, assets, key, sender);
                });
            }
        });
        outer
    }

    // This will check if the asset loader finished loading a specific asset using it's handle
    pub fn has_finished_loading<A: AsyncAsset>(
        &self,
        handle: &AsyncHandle<A>,
    ) -> bool {
        self.loaded.borrow_mut().extend(self.receiver.try_iter());
        self.loaded.borrow().contains(&handle.key)
    }

    // This will wait until the asset referenced by this handle has finished loading
    pub fn wait<A: AsyncAsset>(&self, handle: AsyncHandle<A>) -> Result<A, AssetLoadError> {
        // Spin lock whilst whilst waiting for an asset to load
        while !self.has_finished_loading(&handle) {}

        // Get the global asset queue and find the index of the handle key
        let mut assets = self.assets.write();
        let location = self
            .loaded
            .borrow()
            .iter()
            .position(|k| k == &handle.key)
            .unwrap();

        // Remove the key
        self.loaded.borrow_mut().swap_remove(location);

        // Remove the asset from the global queue and return it
        let boxed = assets.remove(handle.key).unwrap();
        boxed.unwrap().map(|b| *b.downcast::<A>().unwrap())
    }

    // This will wait until all the assets reference by these handles have finished loading
    pub fn wait_from_iter<A: AsyncAsset>(
        &self,
        handles: impl IntoIterator<Item = AsyncHandle<A>>,
    ) -> Vec<Result<A, AssetLoadError>> {
        handles
            .into_iter()
            .map(|handle| self.wait(handle))
            .collect::<Vec<_>>()
    }
}