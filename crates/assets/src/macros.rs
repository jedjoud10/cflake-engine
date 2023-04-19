#[macro_export]
macro_rules! asset {
    ($assets:expr, $file:expr, $prefix:expr) => {
        // If the "CFLAKE_DEBUG_ASSETS" feature is set, then this
        // will load the assets dynamically instead of inserting them into the binary
        cfg_if::cfg_if! {
            if #[cfg(feature = "debug-assets")] {
                let path = concat!(env!("CARGO_MANIFEST_DIR"), $prefix, $file);
                $assets.hijack($file, path);
            } else {
                /*
                with_builtin!(let $bytes = include_bytes_from_root!(concat!("./", $prefix, $file)) in {
                   $assets.import($file, $bytes.to_vec());
                });
                */
            }
        }
    };
}