use io::{LoadableData, LoadedValue, ValueGetter};

// The config file of the world
pub struct GameConfig {
    pub vsync: bool,
}

// Make the config file loadable
impl LoadableData for GameConfig {
    // Load the config struct from the config file
    fn load_from_file(vg: &ValueGetter) -> Self {
        Self {
            vsync: vg.get_bool(0, true),
        }
    }
    // Save to file
    fn save_to_file(&self) -> Vec<io::LoadedValue> {
        return vec![LoadedValue::BOOL(self.vsync)];
    }
}