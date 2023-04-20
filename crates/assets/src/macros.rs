#[macro_export]
macro_rules! asset {
    ($assets:expr, $file:expr, $prefix:expr) => {
        // If the "CFLAKE_PACK_ASSETS" feature is set, then this
        // will pack the asset directly into the binary instead of loading it dynamically
        cfg_if::cfg_if! {
            if #[cfg(feature = "pack-assets")] {
                todo!();
            } else {
                let path = concat!(env!("CARGO_MANIFEST_DIR"), $prefix, $file);
                $assets.hijack($file, path);
            }
        }
    };
}