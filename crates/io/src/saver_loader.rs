use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use platform_dirs::AppDirs;
use serde_json;
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

// Lets us save / load a file from the saved folder
#[derive(Default)]
pub struct SaverLoader {
    // The path where all the local data will be stored into
    pub local_path: Option<PathBuf>,
}

impl SaverLoader {
    // Make sure a default copy of the data exists
    pub fn create_default<T: serde::Serialize + serde::Deserialize<'static>>(&self, file_path: &str, default_data: &T) {
        // If default_create is true, we should create the file if it does not exist yet
        let global_path = self.local_path.as_ref().unwrap().join(file_path);
        if !global_path.exists() {
            let dir_path = global_path.parent().unwrap();
            std::fs::create_dir_all(dir_path).unwrap();
            File::create(global_path.clone()).unwrap();
            self.save(file_path, default_data);
        }
    }
    // Get a new copy of the saver loader
    pub fn new(author_name: &str, app_name: &str) -> Self {
        let old_path = format!("{}\\{}\\", author_name, app_name);
        let path = AppDirs::new(Some(&old_path), false).unwrap();
        println!("Init saver-loader with path: '{:?}'", path.config_dir);
        SaverLoader { local_path: Some(path.config_dir) }
    }
    // Load a struct from a file
    pub fn load<'a, T: serde::Serialize + serde::de::DeserializeOwned>(&self, file_path: &'a str) -> T {
        // Load the file
        let global_path = self.local_path.as_ref().unwrap().join(file_path);
        let reader = BufReader::new(OpenOptions::new().read(true).open(global_path).unwrap());
        let x = serde_json::from_reader(reader).unwrap();
        return x;
    }
    // Save a struct to a file
    pub fn save<T: serde::Serialize + serde::Deserialize<'static>>(&self, file_path: &str, struct_to_save: &T) {
        // Save the file
        let global_path = self.local_path.as_ref().unwrap().join(file_path);
        let mut writer = BufWriter::new(OpenOptions::new().write(true).open(global_path).unwrap());
        let string = serde_json::to_string_pretty(struct_to_save).unwrap();
        writer.write_all(string.as_bytes()).unwrap();
    }
    // Save a string to a specific log file in the local user data
    pub fn save_string(&self, file_path: &str, string: String) {}
}
