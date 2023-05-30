use std::{
    any::TypeId,
    fs::{File, OpenOptions},
    io::{BufReader, Read},
    path::{Path, PathBuf},
    str::FromStr,
};

use ahash::AHashMap;
use platform_dirs::AppDirs;

// Simple input output manager that can read and write from files
// This is very helpful in reducing boilerplate code when reading from config files
pub struct FileManager {
    // App directories that contain the current app data
    dirs: AppDirs,

    // Contains all the vectors that contain Deserialized data
    deserialized: AHashMap<(TypeId, PathBuf), Vec<u8>>,
}

// The type of file that we will read / write
// This will pick where the file should be located
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileType {
    Global,
    Config,
    Data,
    Cache,
    Log,
}

// How exactly we should serialize/deserialize the dat 
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SerdeFormat {
    // Serde will use the JSON formatter to read/write the file
    JSON,

    // Serde will use the Rusty Object Notation formatter to read/write the file
    RON,
}

impl FileManager {
    // Create an IO manager (also creates the AppDirs)
    pub fn new(author: &str, app: &str) -> Self {
        // Create the app path using the author name and app name
        let mut path = PathBuf::from_str(author).unwrap();
        path.push(Path::new(app));

        // Fetch the directory locations
        let path = path.as_os_str().to_str().unwrap();
        let dirs = AppDirs::new(Some(path), false).unwrap();

        // Create the config directory if needed
        Self::init_directory(&dirs.config_dir).unwrap();

        // Create the log directory if needed
        let mut base = dirs.cache_dir.clone();
        base.push("log/");
        Self::init_directory(&base).unwrap();

        Self {
            dirs,
            deserialized: Default::default(),
        }
    }

    // Create a directory in the given path if needed
    pub fn init_directory(path: impl AsRef<Path>) -> std::io::Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            std::fs::DirBuilder::new().recursive(true).create(path)
        } else {
            Ok(())
        }
    }

    // Create an empty file with the given path if needed
    pub fn init_file(path: impl AsRef<Path>) -> std::io::Result<()> {
        if !path.as_ref().exists() {
            let path = path.as_ref();
            std::fs::File::create(path).map(|_| ())
        } else {
            Ok(())
        }
    }

    // Convert a FileType variant and local path to a proper global system path
    fn local_path_to_global(&self, path: impl AsRef<Path>, variant: FileType) -> PathBuf {
        let mut base = match variant {
            FileType::Config => self.dirs.config_dir.clone(),
            FileType::Data => self.dirs.state_dir.clone(),
            FileType::Cache => self.dirs.cache_dir.clone(),
            FileType::Log => {
                let mut base = self.dirs.cache_dir.clone();
                base.push("log/");
                base
            }
            FileType::Global => return path.as_ref().to_path_buf(),
        };

        base.push(path);
        base
    }

    // Create a buf reader for a file
    // PS: This will automatically create the file if needed
    pub fn read_file(&self, path: impl AsRef<Path>, variant: FileType) -> Option<BufReader<File>> {
        // Create the global path based on the variant
        log::debug!("Reading from file {:?}...", path.as_ref());
        let global = self.local_path_to_global(path, variant);

        // Check if there was a file creation error
        let file = Self::init_file(&global);
        let Ok(_) = file else {
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
    pub fn write_file(
        &mut self,
        path: impl AsRef<Path>,
        truncate: bool,
        variant: FileType,
    ) -> Option<File> {
        // So we don't cause a rupture in space time continuum
        if variant != FileType::Log {
            log::debug!("Writing to file {:?}...", path.as_ref());
        }

        // Create the global path
        let global = self.local_path_to_global(path, variant);

        // Check if there was a file creation error
        let file = Self::init_file(&global);
        if let Err(err) = file {
            log::error!("{}", err);
            return None;
        };

        // Create a file write
        let options = OpenOptions::new()
            .append(!truncate)
            .truncate(truncate)
            .write(true)
            .open(global);

        // Check if there was a file write error
        let Ok(file) = options else {
            log::error!("{}", options.err().unwrap());
            return None;
        };

        // Create a file reader
        Some(file)
    }

    // Internally used to deserialize a stream of bytes using a specific format
    fn inner_deserialize_bytes<'a, T: serde::Deserialize<'a> + 'static>(
        bytes: &'a [u8],
        format: SerdeFormat,
    ) -> Option<T> {
        match format {
            SerdeFormat::JSON => {
                let string = core::str::from_utf8(bytes).ok()?;
                let value = serde_json::from_str::<T>(string).ok()?;
                Some(value)
            },
            SerdeFormat::RON => ron::de::from_bytes(bytes).ok(),
        }
    }

    // Internally used to serialize a stream of bytes using a specific format
    fn inner_serialize_bytes<W: std::io::Write, T: serde::Serialize>(
        value: &T,
        writer: W,
        format: SerdeFormat,
    ) -> Option<()> {
        match format {
            SerdeFormat::JSON => {
                serde_json::to_writer_pretty(writer, value).ok()?;
                Some(())
            },
            SerdeFormat::RON => {
                ron::ser::to_writer_pretty(writer, value, ron::ser::PrettyConfig::default()).ok()?;
                Some(())
            },
        }
    }

    // Deserialize a file using Serde and a specific format
    pub fn deserialize_from_file<'a, T: serde::Deserialize<'a> + 'static>(
        &'a mut self,
        path: impl AsRef<Path>,
        variant: FileType,
        format: SerdeFormat,
    ) -> Option<T> {
        // Read the file into a string and then add it internally
        log::debug!("Deserializing to file {:?}...", path.as_ref());
        let mut reader = self.read_file(&path, variant)?;
        let mut buf = Vec::<u8>::new(); 
        reader.read_to_end(&mut buf).unwrap();

        let key = (TypeId::of::<T>(), path.as_ref().to_path_buf());
        let cloned = key.clone();
        self.deserialized.insert(key, buf);
        let last = self.deserialized.get(&cloned).unwrap();

        let value = Self::inner_deserialize_bytes::<T>(&last, format)?;
        log::debug!("Deserialized data from {:?} successfully!", path.as_ref());
        Some(value)
    }

    // Serialize a struct into a file using Serde and a specific format
    pub fn serialize_into_file<T: serde::Serialize>(
        &mut self,
        value: &T,
        path: impl AsRef<Path>,
        variant: FileType,
        format: SerdeFormat,
    ) -> Option<()> {
        log::debug!("Serializing to file {:?}...", path.as_ref());
        
        let writer = self.write_file(&path, true, variant)?;
        Self::inner_serialize_bytes(value, writer, format)?;

        log::debug!("Serialized data into {:?} successfully!", path.as_ref());
        Some(())
    }
}
