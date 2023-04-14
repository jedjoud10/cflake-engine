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
macro_rules! user_assets {
    ($path:expr) => {
        {
            with_builtin!(let $combined_path = concat!(env!("CARGO_MANIFEST_DIR"), $path) in {
                fn fuck_you_macros() -> Vec<(std::path::PathBuf, Vec<u8>)> {
                    static DIR: include_dir::Dir<'_> = include_dir::include_dir!($combined_path);
                    
                    // Keep track of directories we must iterate through
                    let mut dirs = vec![&DIR];

                    // Files that have been loaded in
                    let mut files: Vec<(std::path::PathBuf, Vec<u8>)> = Vec::new();

                    // It's recursing time
                    while let Some(dir) = dirs.pop() {
                        for entry in dir.entries() {
                            match entry {
                                include_dir::DirEntry::Dir(dir) => dirs.push(dir),
                                include_dir::DirEntry::File(file) => {
                                    let path = file.path().to_path_buf();
                                    let content = file.contents().to_vec();
                                    files.push((path, content));
                                },
                            }
                        }
                    }

                    drop(dirs);
                    return files
                }
            });

            // Get the files that the user wishes to load in
            let files = fuck_you_macros();

            // Get the path of the user asset directory
            let path = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), $path));
            let path: std::sync::Arc<std::path::Path> = std::sync::Arc::from(path);

            Some(UserAssets {
                path,
                files,
            })
        }
    };
}
