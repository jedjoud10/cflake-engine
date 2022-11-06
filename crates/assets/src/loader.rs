use crate::Asset;
use ahash::AHashMap;
use parking_lot::RwLock;
use slotmap::{DefaultKey, Key, SlotMap};
use world::ThreadPool;

use std::{
    any::Any,
    cell::{Cell, RefCell},
    ffi::OsStr,
    marker::PhantomData,
    path::{Path, PathBuf},
    rc::Rc,
    str::FromStr,
    sync::{
        atomic::{AtomicU32, Ordering},
        mpsc::{Receiver, Sender},
        Arc,
    },
    thread::Thread,
    time::Instant,
};

// This is a handle to a specific asset that we are currently loading in
pub struct AsyncHandle<'a, A: Asset<'a>> {
    _phantom: PhantomData<&'a A>,
    key: DefaultKey,
}

// This is the main asset manager resource that will load & cache newly loaded assets
// This asset manager will also contain the persistent assets that are included by default into the engine executable
pub struct Assets {
    // Keep track of the assets that were sucessfully loaded
    // The value corresponding to each key might be None in the case that the asset did not load
    assets: Arc<RwLock<SlotMap<DefaultKey, Option<Box<dyn Any + Send + Sync + Sync>>>>>,

    // Keep track of the bytes that were loaded in other threads
    // The value might be none in the case that the bytes were not loaded
    bytes: Arc<RwLock<AHashMap<PathBuf, Arc<[u8]>>>>,

    // This receiver and vec keep track of the key IDs of items that were loaded in
    receiver: Receiver<DefaultKey>,
    sender: Sender<DefaultKey>,
    loaded: RefCell<Vec<DefaultKey>>,

    // Path that references the main user assets
    user: Option<PathBuf>,
}

impl Assets {
    // Create a new asset loader using a path to the user defined asset folder (if there is one)
    pub fn new(user: Option<PathBuf>) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel::<DefaultKey>();

        Self {
            assets: Default::default(),
            bytes: Default::default(),
            loaded: Default::default(),
            receiver,
            sender,
            user,
        }
    }

    // Load an asset using some explicit/implicit loading arguments without checking it's extensions
    pub unsafe fn load_unchecked<'args, A: Asset<'args>>(
        &self,
        path: &str,
        args: A::Args,
    ) -> Option<A> {
        // All this does is that it ensures that the bytes are valid before we actually deserialize the asset
        let path = PathBuf::from_str(path).unwrap();
        let (name, extension) = path
            .file_name()
            .and_then(OsStr::to_str)?
            .split_once('.')
            .unwrap();
        let bytes = if self.bytes.read().contains_key(&path) {
            self.bytes.read().get(&path).unwrap().clone()
        } else {
            // TODO: Proper error logging
            let mut write = self.bytes.write();
            let bytes = super::raw::read(&path, self.user.as_ref()?)?;
            let arc: Arc<[u8]> = Arc::from(bytes);
            write.insert(path.clone(), arc.clone());
            arc
        };

        // Deserialize the asset file
        Some(A::deserialize(
            crate::Data {
                name,
                extension,
                bytes,
                path: &path,
            },
            args,
        ))
    }

    /*
    // Load an asset using some explicit loading arguments
    pub fn load_with<'args, A: Asset<'args>>(&self, path: &str, args: A::Args) -> Option<A> {
        // Check if the extension is valid
        let _path = PathBuf::from_str(path).unwrap();
        let (_, extension) = _path.file_name().and_then(OsStr::to_str)?.split_once('.')?;

        // If the asset has no extensions, we shall not check
        ((A::extensions().contains(&extension)) || A::extensions().is_empty()).then_some(())?;
        unsafe { self.load_with_unchecked(path, args) }
    }

    // Load an asset using some default loading arguments
    pub fn load<'args, A: Asset<'args>>(&self, path: &str) -> Option<A>
    where
        A::Args: Default,
    {
        self.load_with(path, Default::default())
    }

    // Load multiple assets using some explicit loading arguments
    pub fn load_batch_with<'args, A: Asset<'args>>(&self, path: &[&str], args: A::Args) -> Vec<Option<A>> {
        todo!()
    }

    // Load multiple assets using some default loading argument
    pub fn load_batch<'args, A: Asset<'args>>(&self, path: &str) -> Option<A>
    where
        A::Args: Default,
    {
        todo!()
    }


    // Load an asset using some explicit loading arguments without checking it's extensions in another thread
    pub unsafe fn threaded_load_with_unchecked<A: Asset<'static> + Send + Sync>(
        &self,
        path: &str,
        args: A::Args,
        threadpool: &mut ThreadPool,
    ) -> AsyncHandle<'static, A>
    where
        A::Args: Send + Sync,
    {
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

        // Create a multithreaded loading task bozoo
        let path = PathBuf::from_str(path).unwrap();
        threadpool.execute(move || {
            // All this does is that it ensures that the bytes are valid before we actually deserialize the asset
            let (name, extension) = path
                .file_name()
                .and_then(OsStr::to_str)
                .unwrap()
                .split_once('.')
                .unwrap();
            let bytes = if bytes.read().contains_key(&path) {
                bytes.read().get(&path).unwrap().clone()
            } else {
                // TODO: Proper error logging
                let mut write = bytes.write();
                let bytes = super::raw::read(&path, user.as_ref().unwrap()).unwrap();
                let arc: Arc<[u8]> = Arc::from(bytes);
                write.insert(path.clone(), arc.clone());
                arc
            };

            let asset = A::deserialize(
                crate::Data {
                    name,
                    extension,
                    bytes,
                    path: path.as_path(),
                },
                args,
            );

            // Add the deserialized back to the loader
            let mut write = assets.write();
            let opt = write.get_mut(key).unwrap();
            *opt = Some(Box::new(asset));
            sender.send(key).unwrap();
        });

        // Return the async handle
        return handle;
    }

    // Load an asset using some explicit loading arguments in another thread
    pub fn threaded_load_with<A: Asset<'static> + Send + Sync>(
        &self,
        path: &str,
        args: A::Args,
        threadpool: &mut ThreadPool,
    ) -> AsyncHandle<'static, A>
    where
        A::Args: Send + Sync,
    {
        let pathbuf = PathBuf::from_str(path).unwrap();
        let (_, extension) = pathbuf
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap()
            .split_once('.')
            .unwrap();
        ((A::extensions().contains(&extension)) || A::extensions().is_empty())
            .then_some(())
            .unwrap();
        unsafe { self.threaded_load_with_unchecked(path, args, threadpool) }
    }

    // Load an asset using some default loading arguments in another thread
    pub fn threaded_load<A: Asset<'static> + Send + Sync>(
        &self,
        path: &str,
        threadpool: &mut ThreadPool,
    ) -> AsyncHandle<'static, A>
    where
        A::Args: Default + Send + Sync,
    {
        self.threaded_load_with(path, Default::default(), threadpool)
    }

    // This will wait until the asset referenced by this handle has finished loading
    pub fn wait<A: Asset<'static> + Send + Sync>(&self, handle: AsyncHandle<'static, A>) -> A
    where
        A::Args: Send + Sync,
    {
        while !self.was_loaded(&handle) {}
        let mut assets = self.assets.write();
        let boxed = assets.remove(handle.key).unwrap();
        let asset = boxed.unwrap().downcast::<A>().unwrap();
        let location = self
            .loaded
            .borrow()
            .iter()
            .position(|k| k == &handle.key)
            .unwrap();
        self.loaded.borrow_mut().swap_remove(location);
        return *asset;
    }
    */

    // Import a persistent asset using it's global asset path and it's raw bytes
    pub fn import(&self, path: impl AsRef<Path>, bytes: Vec<u8>) {
        let path = path
            .as_ref()
            .strip_prefix("./assets/")
            .unwrap()
            .to_path_buf();
        self.bytes.write().entry(path).or_insert(Arc::from(bytes));
    }
}
