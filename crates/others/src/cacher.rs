use std::{collections::HashMap, fmt};

// A cacher manager struct that can cache any type of data so it doesn't need to be reloaded later on
pub struct CacheManager<A> {
    pub objects: Vec<A>,
    pub names: HashMap<String, usize>,
}

impl<A> Default for CacheManager<A> {
    fn default() -> Self {
        Self {
            objects: Vec::new(),
            names: HashMap::new(),
        }
    }
}

// Custom error handling for the cacher
#[derive(Debug)]
pub struct Error {
    details: String,
}

impl Error {
    pub fn new(msg: &str) -> Self {
        Self { details: msg.to_string() }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.details
    }
}

// Da code
impl<A> CacheManager<A> {
    // Check if an object is cached
    pub fn is_cached(&self, name: &str) -> bool {
        self.names.contains_key(name)
    }
    // Get the ID of an object using it's name
    pub fn get_object_id(&self, name: &str) -> Result<usize, Error> {
        if self.is_cached(name) {
            // That default object name is valid
            let id = self.names[name];
            Ok(id)
        } else {
            // Name's not valid
            return Err(Error::new(format!("Object name '{}' is invalid!", name).as_str()));
        }
    }
    // Cached an object and gives back it's cached ID
    pub fn cache_object(&mut self, object: A, name: &str) -> usize {
        // Cache the object, if it was already cached then just return it's id
        if self.is_cached(name) {
            let id = self.names.get(&name.to_string()).unwrap();
            *id
        } else {
            // The object was never cached, so we've gotta cache it
            self.objects.push(object);
            self.names.insert(name.to_string(), self.objects.len() - 1);

            self.objects.len() - 1
        }
    }
    // Get a reference to an object using it's object name
    pub fn get_object(&self, name: &str) -> Result<&A, Error> {
        if self.is_cached(name) {
            // The object exists, we can safely return it
            return Ok(self.objects.get(self.names[&name.to_string()]).unwrap());
        } else {
            // The object does not exist
            return Err(Error::new(format!("Cached object with name '{}' does not exist!", name).as_str()));
        }
    }
    // Get a reference to an object using it's object name
    pub fn get_object_mut(&mut self, name: &str) -> Result<&mut A, Error> {
        if self.names.contains_key(name) {
            // The object exists, we can safely return it
            return Ok(self.objects.get_mut(self.names[&name.to_string()]).unwrap());
        } else {
            // The object does not exist
            return Err(Error::new(format!("Cached object with name '{}' does not exist!", name).as_str()));
        }
    }
    // Get a reference to an object using it's object ID
    pub fn id_get_object(&self, id: usize) -> Option<&A> {
        self.objects.get(id)
    }
    // Get a reference to an object using it's object ID
    pub fn id_get_object_mut(&mut self, id: usize) -> Option<&mut A> {
        self.objects.get_mut(id)
    }
}
