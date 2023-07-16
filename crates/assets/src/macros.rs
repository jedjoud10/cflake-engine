/// Define an asset path for an asset that will be loaded in.
/// Assets will either be packed into the binary or loaded in at runtime based on the ``pack-assets`` feature
/// # Arguments
///
/// ``assets`` - Immutable asset loader reference
///
/// ``$file`` - Path from the assets folder to the actual asset file
///  
/// ``$prefix`` - Local path towards the assets folder from the root of the crate (``CARGO_MANIFEST_DIR``)
///
#[macro_export]
macro_rules! asset {
    ($assets:expr, $file:expr, $prefix:expr) => {
        // If the "CFLAKE_PACK_ASSETS" feature is set, then this
        // will pack the asset directly into the binary instead of loading it dynamically
        cfg_if::cfg_if! {
            if #[cfg(feature = "pack-assets")] {
                {
                    with_builtin_macros::with_builtin!(let $bytes = include_bytes_from_root!(concat!(
                        env!("CARGO_MANIFEST_DIR"), "/", $prefix, $file,
                    )) in {
                        let path = $file;
                        dbg!(&path);
                        $assets.import(path, $bytes.to_vec());
                    });
                }
            } else {
                let path = concat!(env!("CARGO_MANIFEST_DIR"), $prefix, $file);
                $assets.hijack($file, path);
            }
        }
    };
}
