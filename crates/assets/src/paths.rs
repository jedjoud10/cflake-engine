use std::path::{PathBuf, Path};
use world::Resource;

// This struct contains all the directory paths that we will use
// This will be stored as a resource, and it will be created at the very start of the program automatically
#[derive(Resource)]
pub struct GlobalPaths {
    // This is the directory path for the assets that must be manually loaded at runtime (user assets)
    assets: Option<PathBuf>,

    // Config directory path
    config: PathBuf,
    
    // Log directory path
    log: PathBuf
}

impl GlobalPaths {
    // Get the user assets directory path
    pub fn user_assets_dir(&self) -> Option<&Path> {
        self.assets.as_ref().map(core::convert::AsRef::as_ref)
    }
    
    // Get the game engine config directory path
    pub fn config_dir(&self) -> &Path {
        &self.config
    }
    
    // Get the game engine log directory path
    pub fn log_dir(&self) -> &Path {
        &self.log
    }
}