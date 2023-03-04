#[macro_export]
macro_rules! asset {
    ($assets:expr, $file:expr) => {
        // Include an asset into the binary if we are in release
        #[cfg(not(debug_assertions))]
        {
            let bytes = include_bytes!($file);
            $assets.import(concat!("./", $file), bytes.to_vec());
        }
        // Don't do anything in debug since it'll read it from the file system
        {
            let x = &mut $assets;
        }
    };
}

#[macro_export]
macro_rules! persistent {
    ($assets:expr, $file:expr) => {
        // If the "CFLAKE_DEBUG_ASSETS" environment variable is set, then this
        // will load the assets dynamically instead of inserting them into the binary
        match crate::raw::engine_debug_assets_enabled() {
            true => {
                let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/assets/", $file);
                $assets.hijack($file, path);
            },
            false => {
                let bytes = include_bytes!(concat!("./assets/", $file));
                $assets.import(concat!("./assets/", $file), bytes.to_vec());
            }
        }
    };
}

#[macro_export]
macro_rules! user_assets_path {
    ($suffix:expr) => {
        concat!(env!("CARGO_MANIFEST_DIR"), $suffix);
    };
}
