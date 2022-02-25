#[macro_export]
macro_rules! init {
    ($asset_dir_path:expr) => {
        let mut cacher = $crate::cacher::cacher();
        cacher.init(concat!(env!("CARGO_MANIFEST_DIR"), $asset_dir_path));
        drop(cacher);
    };
}


#[macro_export]
macro_rules! asset {
    ($file:expr) => {
        // Include an asset into the binary if we are in release
        #[cfg(not(debug_assertions))]
        {
            let bytes = include_bytes!($file);
            let mut cacher = $crate::cacher::cacher();
            cacher.cache_persistent($file, bytes.to_vec());
            drop(cacher);
        }
        // Don't do anything in debug since it'll read it from the file system
    };
}

#[macro_export]
macro_rules! persistent {
    ($file:expr) => {
        let bytes = include_bytes!($file);
        let mut cacher = $crate::cacher::cacher();
        cacher.cache_persistent($file, bytes.to_vec());
        drop(cacher);
    };
}