use crate::{Asset, AssetInput, AsyncAsset};
use ahash::AHashMap;
use parking_lot::RwLock;
use slotmap::{DefaultKey, SlotMap};
use utils::ThreadPool;

use std::{
    any::Any,
    cell::RefCell,
    ffi::OsStr,
    marker::PhantomData,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
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
    assets: Arc<
        RwLock<
            SlotMap<
                DefaultKey,
                Option<Box<dyn Any + Send + Sync + Sync>>,
            >,
        >,
    >,

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
        let (sender, receiver) =
            std::sync::mpsc::channel::<DefaultKey>();

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

// Synchronous loading
impl Assets {
    // Check if the extension of a file is valid
    fn is_extension_valid<A: Asset>(path: &PathBuf) -> Option<()> {
        let (_, extension) = path
            .file_name()
            .and_then(OsStr::to_str)?
            .split_once('.')?;

        // If the asset has no extensions, we shall not check
        ((A::extensions().contains(&extension))
            || A::extensions().is_empty())
        .then_some(())
    }

    // Load an asset using some explicit/default loading arguments
    pub fn load<'s, 'args, A: Asset>(
        &self,
        input: impl AssetInput<'s, 'args, A>,
    ) -> Option<A> {
        // Check if the extension is valid
        let (path, args) = input.split();
        let path = PathBuf::from_str(path).unwrap();
        Self::is_extension_valid::<A>(&path)?;

        // All this does is that it ensures that the bytes are valid before we actually deserialize the asset
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

    // Load multiple assets using some explicit/default loading arguments
    pub fn load_from_iter<'s, 'args, A: Asset>(
        &self,
        inputs: impl IntoIterator<Item = impl AssetInput<'s, 'args, A>>,
    ) -> Vec<Option<A>> {
        inputs
            .into_iter()
            .map(|input| self.load(input))
            .collect::<Vec<Option<A>>>()
    }
}

// Asynchronous loading
impl Assets {
    // Load an asset using some explicit/default loading arguments in another thread
    pub fn async_load<'s, A: AsyncAsset>(
        &self,
        input: impl AssetInput<'s, 'static, A>,
        threadpool: &mut ThreadPool,
    ) -> Option<AsyncHandle<A>>
    where
        A::Args<'static>: Send + Sync,
    {
        // Get the path and arguments
        let (path, args) = input.split();
        let path = PathBuf::from_str(path).unwrap();
        Self::is_extension_valid::<A>(&path)?;

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
                let bytes =
                    super::raw::read(&path, user.as_ref().unwrap())
                        .unwrap();
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
        Some(handle)
    }

    // Load multiple assets using some explicit/default loading arguments in another thread
    // This returns handle(s) that we can wait for and fetch later on
    pub fn async_load_from_iter<'s, A: AsyncAsset>(
        &self,
        inputs: impl IntoIterator<
            Item = impl AssetInput<'s, 'static, A> + Send,
        >,
        threadpool: &mut ThreadPool,
    ) -> Vec<Option<AsyncHandle<A>>> {
        // Create a temporary threadpool scope for these assets only
        let mut outer = Vec::<Option<AsyncHandle<A>>>::new();
        let reference = &mut outer;
        threadpool.scope(move |scope| {
            for input in inputs.into_iter() {
                // Check the extension on a per file basis
                let path = PathBuf::from_str(input.path()).unwrap();
                if Self::is_extension_valid::<A>(&path).is_none() {
                    reference.push(None);
                    continue;
                }

                // Clone the things that must be sent to the thread
                let assets = self.assets.clone();
                let bytes = self.bytes.clone();
                let sender = self.sender.clone();
                let user = self.user.clone();

                // Create the handle's key and insert it
                let key = self.assets.write().insert(None);
                let handle = AsyncHandle::<A> {
                    _phantom: PhantomData,
                    key,
                };
                reference.push(Some(handle));

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
                        let bytes = super::raw::read(
                            &path,
                            user.as_ref().as_ref().unwrap(),
                        )
                        .unwrap();
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
            }
        });
        outer
    }

    // This will check if the asset loader finished loading a specific asset using it's handle
    pub fn has_finished_loading<A: AsyncAsset>(
        &self,
        handle: &AsyncHandle<A>,
    ) -> bool {
        // Poll and update
        self.loaded.borrow_mut().extend(self.receiver.try_iter());

        // Check
        self.loaded.borrow().contains(&handle.key)
    }

    // This will wait until the asset referenced by this handle has finished loading
    pub fn wait<A: AsyncAsset>(&self, handle: AsyncHandle<A>) -> A {
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
        *boxed.unwrap().downcast::<A>().unwrap()
    }

    // This will wait until all the assets reference by these handles have finished loading
    pub fn wait_from_iter<A: AsyncAsset>(
        &self,
        handles: impl IntoIterator<Item = AsyncHandle<A>>,
    ) -> Vec<A> {
        handles
            .into_iter()
            .map(|handle| self.wait(handle))
            .collect::<Vec<_>>()
    }
}
