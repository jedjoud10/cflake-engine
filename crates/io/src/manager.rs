use log::LevelFilter;
use parking_lot::Mutex;
use platform_dirs::AppDirs;

use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::logger::Logger;

// Lets us save / load a file from the saved folder
#[derive(Default)]
pub struct Manager {
    // The path where all the local data will be stored into
    pub local_path: Option<PathBuf>,
    // An arc containing all the logged messages
    messages: Arc<Mutex<Vec<String>>>,
    log_file_path: PathBuf,
}

impl Manager {
    // Get a new copy of the saver loader
    pub fn new(author_name: &str, app_name: &str) -> Self {
        let old_path = format!("{}/{}/", author_name, app_name);
        let path = AppDirs::new(Some(&old_path), false).unwrap();
        println!("Init saver-loader with path: '{:?}'", path.config_dir);
        // Also init the logger
        let log_file_path = {
            let mut path = path.config_dir.clone();
            path.push("log/recent.log");

            // Make sure the log file exists
            let parent = path.parent().unwrap();
            if !path.exists() {
                std::fs::create_dir_all(parent).unwrap();
                File::create(path.clone()).unwrap();
            }
            path
        };
        let messages = Arc::new(Mutex::new(Vec::new()));
        let logger = Logger {
            messages: messages.clone(),
        };
        log::set_boxed_logger(Box::new(logger)).unwrap();
        log::set_max_level(LevelFilter::Info);
        Manager {
            local_path: Some(path.config_dir),
            messages,
            log_file_path,
        }
    }
    // Close everything and stop the saver loader
    pub fn quit(&self) {
        // Mass write
        let mut lock = self.messages.lock();
        let taken = lock.drain(..);
        // Open the log file so we can start writing to it
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(&self.log_file_path)
            .unwrap();
        let mut writer = BufWriter::new(file);
        for message in taken {
            writeln!(&mut writer, "{}", message).unwrap();
        }
    }
    // Create a file if it is not created yet
    pub fn create_file(&self, file_path: impl AsRef<Path>) {
        let mut path = self.local_path.clone().unwrap();
        path.push(file_path);
        let parent = path.parent().unwrap();
        if !path.exists() {
            std::fs::create_dir_all(parent).unwrap();
            File::create(path).unwrap();
        }
    }
    // Load a struct from a file
    pub fn load<T: serde::Serialize + serde::de::DeserializeOwned>(
        &self,
        file_path: impl AsRef<Path>,
    ) -> io::Result<T> {
        // Load the file
        let global_path = self.local_path.as_ref().unwrap().join(file_path);
        let reader = BufReader::new(OpenOptions::new().read(true).open(global_path)?);

        Ok(serde_json::from_reader(reader).unwrap())
    }
    // Save a struct to a file
    pub fn save<T: serde::Serialize + serde::Deserialize<'static>>(
        &self,
        file_path: impl AsRef<Path>,
        struct_to_save: &T,
    ) {
        // Save the file
        let global_path = self.local_path.as_ref().unwrap().join(file_path);
        let mut writer = BufWriter::new(OpenOptions::new().write(true).open(global_path).unwrap());
        let string = serde_json::to_string_pretty(struct_to_save).unwrap();
        writer.write_all(string.as_bytes()).unwrap();
    }
}
