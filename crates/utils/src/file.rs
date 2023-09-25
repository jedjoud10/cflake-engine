use std::{
    fs::{File, OpenOptions},
    io::{BufReader},
    path::{Path, PathBuf},
    str::FromStr,
};

use platform_dirs::AppDirs;

// Simple input output manager that can read and write from files
// This is very helpful in reducing boilerplate code when reading from config files
pub struct FileManager {
    // App directories that contain the current app data
    dirs: AppDirs,
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
}
