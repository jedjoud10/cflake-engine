use std::{fs::{File, OpenOptions}, io::{BufReader, BufWriter, Seek}};

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};

// An enum value that can will be used when loading / saving structs 
pub enum LoadValue {
    None,
    BOOL(bool),
    F32(f32),
    I32(i32),
}

// A trait that will be implemented on structs that can be turned into .hsf files
pub trait Loadable {
    fn load_from_file(values: &Vec<LoadValue>) -> Self;
    fn save_to_file(&self) -> Vec<LoadValue>;
}

// Lets us save / load a file from the saved folder
pub struct SaverLoader {
    // The path where all the local data will be stored into
    // %appdata%\\{game_name}\\data\\
    pub local_path: String,
}

impl SaverLoader {
    // Load a struct from a file
    pub fn load<T: Loadable>(&self, file_path: &str) -> T {
        // Load the file
        let global_path = format!("{}\\{}", self.local_path, file_path);
        let mut reader = BufReader::new(OpenOptions::new().read(true).open(global_path).unwrap());
        let cap = reader.buffer();
        let mut values: Vec<LoadValue> = Vec::new();
        // Get the values from this reader
        loop {
            // Get the value type 
            let _type = match reader.read_u8() {
                Ok(x) => x as i32,
                Err(_) => {
                    // Quit from the loop
                    break;
                },
            };
            let value_to_add = match _type {
                0 => {
                    // bool
                    LoadValue::BOOL(match reader.read_u8().unwrap() {
                        0 => false,
                        255 => true,
                        _ => panic!()
                    })
                }
                1 => {
                    // f32
                    LoadValue::F32(reader.read_f32::<LittleEndian>().unwrap())
                }
                2 => {
                    // i32
                    LoadValue::I32(reader.read_i32::<LittleEndian>().unwrap())
                }
                _ => panic!()
            };
            values.push(value_to_add);            
        }
        let v = T::load_from_file(&values);
        return v;
    }
    // Save a struct to a file
    pub fn save<T: Loadable>(&self, file_path: &str, struct_to_save: T) {
        // Save the file
        let global_path = format!("{}\\{}", self.local_path, file_path);
        let mut writer = BufWriter::new(OpenOptions::new().write(true).open(global_path).unwrap());
        let values = struct_to_save.save_to_file();
        // Actually save the file

    }
}