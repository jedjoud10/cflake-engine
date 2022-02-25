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