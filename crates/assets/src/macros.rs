#[macro_export]
macro_rules! preload_asset {
    ($file:expr) => {
        let bytes = include_bytes!($file);
        let mut cacher = crate::main::ASSETM.lock().unwrap();
        cacher.pre_load($file, bytes).unwrap();
        drop(cacher);
    };
}
