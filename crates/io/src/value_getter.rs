use crate::{LoadedValue};

// A struct containing all the loaded values, and a simple function to get them back
pub struct ValueGetter {
    pub values: Vec<LoadedValue>
}

// Get the values, or return a default value instead
impl ValueGetter {
    // Return the updated loaded value or just return the original value
    pub fn get_bool(&self, index: usize, default_value: bool) -> bool {
        match self.values.get(index) {
            Some(x) => {
                match x {
                    LoadedValue::BOOL(b) => *b,
                    _ => { /* Nothing */ panic!("Value loaded at index {} is not a bool!", index) }
                }
            },
            None => { /* No value loaded, return the original */ return default_value; },
        }
    }
}