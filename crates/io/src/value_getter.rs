use crate::{LoadedValue};

// A struct containing all the loaded values, and a simple function to get them back
pub struct ValueGetter {
    pub values: Vec<LoadedValue>,
    pub current_cursor_index: usize,

    // Returns true if the loaded values already exists, return false if we have ANY missing values
    pub valid: bool,
}

// Get the values, or return a default value instead
impl ValueGetter {
    // Return the updated loaded value or just return the original value
    pub fn get_bool(&mut self, default_value: bool) -> bool {
        let output = match self.values.get(self.current_cursor_index) {
            Some(x) => {
                match x {
                    LoadedValue::BOOL(b) => *b,
                    _ => { /* Nothing */ panic!("Value loaded at index {} is not a bool!", self.current_cursor_index) }
                }
            },
            None => { 
                // No value loaded, return the original
                self.valid = false;            
                return default_value;            
            },
        };
        self.current_cursor_index+= 1;
        return output;
    }
}