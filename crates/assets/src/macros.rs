#[macro_export]
macro_rules! preload_asset {
    ($file:expr, $cacher:expr) => {
        let bytes = include_bytes!($file);
        $cacher.pre_load($file, bytes).unwrap();
    };
}
