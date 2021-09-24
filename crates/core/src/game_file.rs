use io::{LoadableData, LoadedValue, ValueGetter};

// The config file of the world
pub struct GameConfig {
    pub vsync: bool,
    pub fullscreen: bool,
}

// Default
impl Default for GameConfig {
    fn default() -> Self {
        Self { 
            vsync: true,
            fullscreen: true,
        }
    }
}

// Make the config file loadable
impl LoadableData for GameConfig {
    // Load the config struct from the config file
    fn load_from_file(vg: &mut ValueGetter) -> Self {
        let default_self = Self::default();
        Self {
            vsync: vg.get_bool( default_self.vsync),
            fullscreen: vg.get_bool(default_self.fullscreen),
        }
    }
    // Save to file
    fn save_to_file(&self) -> Vec<io::LoadedValue> {
        return vec![
            LoadedValue::BOOL(self.vsync),
            LoadedValue::BOOL(self.fullscreen)
        ];
    }
}