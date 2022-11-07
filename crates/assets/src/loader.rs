use crate::{Asset, AssetInput};
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
pub struct AsyncHandle<A: Asset> {
    _phantom: PhantomData<A>,
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

    // Load an asset using some explicit/default loading arguments without checking it's extensions
    pub unsafe fn load_unchecked<'s, 'args, A: Asset>(
        &self,
        input: impl AssetInput<'s, 'args, A>,
    ) -> Option<A> {
        let (path, args) = input.split();
        
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

    // Load an asset using some explicit/default loading arguments
    pub fn load<'s, 'args, A: Asset>(&self, input: impl AssetInput<'s, 'args, A>) -> Option<A> {
        // Check if the extension is valid
        let path = PathBuf::from_str(input.path()).unwrap();
        let (_, extension) = path.file_name().and_then(OsStr::to_str)?.split_once('.')?;
    
        // If the asset has no extensions, we shall not check
        ((A::extensions().contains(&extension)) || A::extensions().is_empty()).then_some(())?;
        unsafe { self.load_unchecked(input) }
    }

    // Load an asset using some explicit/default loading arguments without checking it's extensions in another thread
    // This requires the arguments of the asset to live as long as 'static
    pub unsafe fn async_load_unchecked<'s, A: Asset + Send + Sync>(
        &self,
        input: impl AssetInput<'s, 'static, A>,
        threadpool: &mut ThreadPool,
    ) -> AsyncHandle<A>
    where
        A::Args<'static>: Send + Sync,
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
        
        // Get the path and arguments
        let (path, args) = input.split();

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

    // Load an asset using some explicit/default loading arguments in another thread
    // This requires the arguments of the asset to live as long as 'static
    pub fn async_load<'s, A: Asset + Send + Sync>(
        &self,
        input: impl AssetInput<'s, 'static, A>,
        threadpool: &mut ThreadPool,
    ) -> Option<AsyncHandle<A>>
    where
        A::Args<'static>: Send + Sync,
    {
        let pathbuf = PathBuf::from_str(input.path()).unwrap();
        let (_, extension) = pathbuf
            .file_name()
            .and_then(OsStr::to_str)?
            .split_once('.')?;
        ((A::extensions().contains(&extension)) || A::extensions().is_empty())
            .then_some(())?;
        Some(unsafe { self.async_load_unchecked(input, threadpool) })
    }

    // This will check if the asset loader finished loading a specific asset using it's handle
    // This requires the arguments of the asset to live as long as 'static
    pub fn was_loaded<A: Asset + Send + Sync>(
        &self,
        handle: &AsyncHandle<A>,
    ) -> bool
    where A::Args<'static>: Send + Sync,
    {
        self.loaded.borrow_mut().extend(self.receiver.try_iter());
        self.loaded.borrow().contains(&handle.key)
    }

    // This will wait until the asset referenced by this handle has finished loading
    // This requires the arguments of the asset to live as long as 'static
    pub fn wait<A: Asset + Send + Sync>(&self, handle: AsyncHandle<A>) -> A
    where A::Args<'static>: Send + Sync,
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

    // Load multiple assets that have the same type in multiple threads at the same time without checking their extensions
    // This does *not* require the arguments of the asset to live as long as 'static
    pub unsafe fn batch_async_load_unchecked<'s, 'args, A: Asset + Send + Sync>(
        &self,
        inputs: Vec<impl AssetInput<'s, 'args, A> + Send + Sync>,
        threadpool: &mut ThreadPool,
    ) -> Vec<A>
    where
        A::Args<'args>: Send + Sync,
    {
        // Create specialized sender/receiver channel just for this asset
        let (tx, rx) = std::sync::mpsc::channel::<(usize, A)>();
        let bytes = self.bytes.clone();
        let user = Arc::new(self.user.clone());

        // Create a temporary threadpool scope for these assets only
        threadpool.scope(move |scope| {
            for (i, input) in inputs.into_iter().enumerate() {
                // These must be sent to other threads
                let tx = tx.clone();
                let user = user.clone();
                let bytes = bytes.clone();
                
                scope.execute(move || {
                    // All this does is that it ensures that the bytes are valid before we actually deserialize the asset
                    let (path, args) = input.split();
                    let path = PathBuf::from_str(path).unwrap();
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
                        let bytes = super::raw::read(&path, user.as_ref().as_ref().unwrap()).unwrap();
                        let arc: Arc<[u8]> = Arc::from(bytes);
                        write.insert(path.clone(), arc.clone());
                        arc
                    };
                    
                    // Deserialize the asset and send it back
                    tx.send((i, A::deserialize(
                        crate::Data {
                            name,
                            extension,
                            bytes,
                            path: path.as_path(),
                        },
                        args,
                    ))).unwrap();
                });
            }
        });

        // Sort the assets that were given from the other threads in the original order they were in
        let mut vec = rx.into_iter().collect::<Vec<(usize, A)>>();
        vec.sort_unstable_by(|(i1, _), (i2, _)| usize::cmp(i1, i2));
        vec.into_iter().map(|(_, a)| a).collect::<Vec<A>>()
    }

    // Load multiple assets that have the same type in multiple threads at the same time
    // This does *not* require the arguments of the asset to live as long as 'static
    pub fn batch_async_load_<'s, 'args, A: Asset + Send + Sync>(
        &self,
        mut input: Vec<impl AssetInput<'s, 'args, A> + Send + Sync>,
        threadpool: &mut ThreadPool,
    ) -> Vec<Option<A>>
    where
        A::Args<'args>: Send + Sync,
    {
        // Closure that must be called for each asset before we load it in another thread
        let closure = |path: &str| {
            let pathbuf = PathBuf::from_str(path).unwrap();
            let (_, extension) = pathbuf
            .file_name()
            .and_then(OsStr::to_str)?
            .split_once('.')?;
            ((A::extensions().contains(&extension)) || A::extensions().is_empty())
            .then_some(())?;
            Some(())
        };

        // Filter out invalid assets because of their extensions
        input.retain(|input| {
            closure(input.path()).is_some()
        });

        // Remap Asset to Option<Asset>
        unsafe { self.batch_async_load_unchecked(input, threadpool) }.into_iter().map(Some).collect()
    }
    

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
