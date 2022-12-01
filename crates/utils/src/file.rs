use std::{path::{PathBuf, Path}, str::FromStr, io::{BufReader, BufWriter}, fs::{File, OpenOptions}};

use platform_dirs::AppDirs;

// Simple input output manager that can read and write from files
// This is very helpful in reducing boilerplate code when reading from config files
pub struct FileManager {
    // App directories that contain the current app data
    dirs: AppDirs,

    // Contains all the strings that we have loaded from files
    strings: Vec<String>
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
            strings: Vec::new(),
        }
    }

    // Create a directory in the given path if needed
    pub fn initialize_directory(path: impl AsRef<Path>) -> std::io::Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            std::fs::DirBuilder::new().recursive(true).create(path)
        } else {
            Ok(())
        }
    }
    
    // Create an empty file with the given path if needed
    pub fn initialize_file(path: impl AsRef<Path>) -> std::io::Result<File> {
        let path = path.as_ref();
        std::fs::File::create(path)
    }

    // Create a buf reader for a file
    // PS: This will automatically create the file if needed
    pub fn read(&self, path: impl AsRef<Path>) -> std::io::Result<BufReader<File>> {
        // Create the global path 
        let mut global = self.dirs.config_dir.clone();
        global.push(path);
        Self::initialize_file(&global)?;

        // Create a file reader
        let options = OpenOptions::new().read(true).open(global)?;
        Ok(BufReader::new(options))
    }

    // Creat a buf write and write data to a file
    // PS: This will automatically create the file if needed
    pub fn write(&self, path: impl AsRef<Path>) -> std::io::Result<BufWriter<File>> {
        // Create the global path 
        let mut global = self.dirs.config_dir.clone();
        global.push(path);      
        Self::initialize_file(&global)?;

        // Create a file reader
        let options = OpenOptions::new().write(true).append(true).open(global)?;
        Ok(BufWriter::new(options))
    }

    // Deserialize a file using Serde
    // Serialize a struct into a file using Serde
}