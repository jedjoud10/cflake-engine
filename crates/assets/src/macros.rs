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

/*
        {
       	    /*
            with_builtin!(let $cock = concat!(env!("CARGO_MANIFEST_DIR"), $path) in {
                include_dir!($cock)
            });
            */           
	    
            // idk how to do this shit bro pls help
            // include_dir! requires a string literal, but we can only combine strings to make a string slice   
            Some(UserAssets {
                path: todo!(),
                files: todo!(),
            })
        }
 */

#[macro_export]
macro_rules! user_assets {
    ($path:expr) => {
        None
    };
}
