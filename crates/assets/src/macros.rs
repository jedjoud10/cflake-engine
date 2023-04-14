#[macro_export]
macro_rules! persistent {
    ($assets:expr, $file:expr) => {
        // If the "CFLAKE_DEBUG_ASSETS" feature is set, then this
        // will load the assets dynamically instead of inserting them into the binary
        cfg_if::cfg_if! {
            if #[cfg(feature = "debug-assets")] {
                let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/assets/", $file);
                $assets.hijack($file, path);
            } else {
                let bytes = include_bytes!(concat!("./assets/", $file));
                $assets.import(concat!("./assets/", $file), bytes.to_vec());
            }
        }
    };
}

#[macro_export]
macro_rules! assets {
    ($suffix:expr) => {
        
    };
}
