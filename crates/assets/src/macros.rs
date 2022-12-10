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
        let bytes = include_bytes!(concat!("./assets/", $file));
        $assets.import(concat!("./assets/", $file), bytes.to_vec());
    };
}

#[macro_export]
macro_rules! user_assets_path {
    ($suffix:expr) => {
        concat!(env!("CARGO_MANIFEST_DIR"), $suffix);
    };
}
