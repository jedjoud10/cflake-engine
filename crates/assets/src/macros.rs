#[macro_export]
macro_rules! preload_asset {
    ($file:expr) => {
        let bytes = include_bytes!($file);
        let mut cacher = crate::main::asset_cacher();
        cacher.pre_load($file, bytes).unwrap();
    };
}
