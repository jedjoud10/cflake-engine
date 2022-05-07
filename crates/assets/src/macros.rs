#[macro_export]
macro_rules! asset {
    ($cacher:expr, $file:expr) => {
        // Include an asset into the binary if we are in release
        #[cfg(not(debug_assertions))]
        {
            let bytes = include_bytes!($file);
            cacher.cache_persistent($file, bytes.to_vec());
        }
        // Don't do anything in debug since it'll read it from the file system
    };
}

#[macro_export]
macro_rules! persistent {
    ($cacher:expr, $file:expr) => {
        let bytes = include_bytes!($file);
        cacher.cache_persistent($file, bytes.to_vec());
    };
}
