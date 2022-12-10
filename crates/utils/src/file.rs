use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read},
    path::{Path, PathBuf},
    str::FromStr, any::TypeId,
};

use ahash::AHashMap;
use platform_dirs::AppDirs;

// Simple input output manager that can read and write from files
// This is very helpful in reducing boilerplate code when reading from config files
pub struct FileManager {
    // App directories that contain the current app data
    dirs: AppDirs,

    // Contains all the strings that contain Deserialized data (because serde's a bit weird)
    strings: AHashMap<TypeId, String>,
}

impl FileManager {
    // Create an IO manager
    pub fn new(author: &str, app: &str) -> Self {
        // Create the app path using the author name and app name
        let mut path = PathBuf::from_str(author).unwrap();
        path.push(Path::new(app));

        // Fetch the directory locations
        let path = path.as_os_str().to_str().unwrap();
        let dirs = AppDirs::new(Some(path), false).unwrap();

        // Create the config directory if needed
        Self::initialize_directory(&dirs.config_dir).unwrap();

        Self {
            dirs,
            strings: Default::default(),
        }
    }

    // Create a directory in the given path if needed
    pub fn initialize_directory(
        path: impl AsRef<Path>,
    ) -> std::io::Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            std::fs::DirBuilder::new().recursive(true).create(path)
        } else {
            Ok(())
        }
    }

    // Create an empty file with the given path if needed
    pub fn initialize_file(
        path: impl AsRef<Path>,
    ) -> std::io::Result<File> {
        let path = path.as_ref();
        std::fs::File::create(path)
    }

    // Create a buf reader for a file
    // PS: This will automatically create the file if needed
    pub fn read(
        &self,
        path: impl AsRef<Path>,
    ) -> Option<BufReader<File>> {
        // Create the global path
        log::debug!("Reading from file {:?}...", path.as_ref());
        let mut global = self.dirs.config_dir.clone();
        global.push(path);

        // Check if there was a file creation error
        let file = Self::initialize_file(&global);
        let Ok(mut file) = file else {
            log::error!("{}", file.err().unwrap());
            return None;
        };

        // Create a file reader
        let options = OpenOptions::new().read(true).open(global);

        // Check if there was a file options error
        let Ok(file) = options else {
            log::error!("{}", options.err().unwrap());
            return None;
        };

        Some(BufReader::new(file))
    }

    // Creat a buf write and write data to a file
    // PS: This will automatically create the file if needed
    pub fn write(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Option<BufWriter<File>> {
        // Create the global path
        log::debug!("Writing to file {:?}...", path.as_ref());
        let mut global = self.dirs.config_dir.clone();
        global.push(path);

        // Check if there was a file creation error
        let file = Self::initialize_file(&global);
        let Ok(file) = file else {
            log::error!("{}", file.err().unwrap());
            return None;
        };

        // Create a file write
        let options = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(global);

        // Check if there was a file write error
        let Ok(file) = options else {
            log::error!("{}", options.err().unwrap());
            return None;
        };

        // Create a file reader
        Some(BufWriter::new(file))
    }

    // Deserialize a file using Serde
    pub fn deserialize<'a, T: serde::Deserialize<'a> + 'static>(
        &'a mut self,
        path: impl AsRef<Path>,
    ) -> Option<T> {
        // Read the file into a string and then add it internally
        log::debug!("Deserializing to file {:?}...", path.as_ref());
        let mut reader = self.read(&path)?;
        let mut string = String::new();
        reader.read_to_string(&mut string).ok()?;
        self.strings.insert(TypeId::of::<T>(), string);
        let last = self.strings.get(&TypeId::of::<T>()).unwrap();
        let value = serde_json::from_str(last.as_str()).unwrap();
        log::debug!(
            "Deserialized data from {:?} successfully!",
            path.as_ref()
        );
        Some(value)
    }

    // Serialize a struct into a file using Serde
    pub fn serialize<T: serde::Serialize>(
        &mut self,
        value: &T,
        path: impl AsRef<Path>,
    ) -> Option<()> {
        log::debug!("Serializing to file {:?}...", path.as_ref());
        let writer = self.write(&path)?;
        serde_json::to_writer_pretty(writer, value).ok()?;
        log::debug!(
            "Serialized data into {:?} successfully!",
            path.as_ref()
        );
        Some(())
    }
}
