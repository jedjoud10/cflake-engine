use platform_dirs::AppDirs;

use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

// Lets us save / load a file from the saved folder
#[derive(Default)]
pub struct IOManager {
    // The path where all the local data will be stored into
    pub local_path: Option<PathBuf>,
}

impl IOManager {
    // Get a new copy of the saver loader
    pub fn new(title: &str) -> Self {
        let path = AppDirs::new(Some(title), false).unwrap();
        println!("Init saver-loader with path: '{:?}'", path.config_dir);
        IOManager {
            local_path: Some(path.config_dir),
        }
    }
    // Create a file relative to the game's data folder
    pub fn create_file(&self, file_path: impl AsRef<Path>) {
        let mut path = self.local_path.clone().unwrap();
        path.push(file_path);
        let parent = path.parent().unwrap();
        if !path.exists() {
            std::fs::create_dir_all(parent).unwrap();
            File::create(path).unwrap();
        }
    }
    // Load a file relative to the game's data folder
    pub fn open_file(&self, file_path: impl AsRef<Path>, options: &OpenOptions) -> io::Result<File> {
        let mut path = self.local_path.clone().unwrap();
        path.push(file_path);
        options.open(path)
    }
    // Load a struct from a file
    pub fn load<T: serde::Serialize + serde::de::DeserializeOwned>(&self, file_path: impl AsRef<Path>) -> io::Result<T> {
        // Load the file
        let global_path = self.local_path.as_ref().unwrap().join(file_path);
        let reader = BufReader::new(OpenOptions::new().read(true).open(global_path)?);

        Ok(serde_json::from_reader(reader).unwrap())
    }
    // Save a struct to a file
    pub fn save<T: serde::Serialize + serde::Deserialize<'static>>(&self, file_path: impl AsRef<Path>, struct_to_save: &T) {
        // Save the file
        let global_path = self.local_path.as_ref().unwrap().join(file_path);
        let mut writer = BufWriter::new(OpenOptions::new().write(true).open(global_path).unwrap());
        let string = serde_json::to_string_pretty(struct_to_save).unwrap();
        writer.write_all(string.as_bytes()).unwrap();
    }
}
