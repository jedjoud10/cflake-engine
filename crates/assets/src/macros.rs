#[macro_export]
macro_rules! cache_persistent {
    ($file:expr) => {
        let bytes = include_bytes!($file);
        let mut cacher = $crate::cacher::cacher();
        cacher.cache_persistent($file, bytes.to_vec());
        drop(cacher);
    };
}
