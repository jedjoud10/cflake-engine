use io::Loadable;

// The config file of the world
pub struct ConfigFile {
    pub vsync: bool,
}

// Make the config file loadable
impl Loadable for ConfigFile {
    // Load the config file
    fn load_from_file(values: &Vec<io::LoadValue>) -> Self {
        let new = ConfigFile {
            vsync: values.get_value_or_default()
        }
        return new;
    }
    // Save the config file
    fn save_to_file(&self) -> Vec<io::LoadValue> {
        todo!()
    }
}