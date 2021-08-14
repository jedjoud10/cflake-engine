use std::{collections::HashMap, fmt};

// A cacher manager struct that can cache any type of data so it doesn't need to be reloaded later on
pub struct CacheManager<A> {
	pub objects: Vec<A>,
	pub names: HashMap<String, u16>,
	pub defaults: Vec<String>
}

impl<A> Default for CacheManager<A> {
    fn default() -> Self {
        Self {
			objects: Vec::new(),
			names: HashMap::new(),
			defaults: Vec::new(),
		}
    }
}

// Custom error handling for the cacher
#[derive(Debug)]
pub struct Error {
    details: String
}

impl Error {
    pub fn new(msg: &str) -> Self {
        Self{details: msg.to_string()}
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
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
	// Generate default objects maps so we can use them late
	pub fn generate_defaults(&mut self, mut default_objects_names: Vec<&str>) {		
		let mut default_object_names: Vec<String> = default_objects_names.iter().map(|&x| x.to_string()).collect();
		self.defaults.append(&mut default_object_names);
	}
	// Get a default object using it's ID
	pub fn id_get_default_object(&self, id: u16) -> Result<&A, Error> {
		if id < self.defaults.len() as u16 {
			// The ID is valid
			let name = self.defaults[id as usize].clone();
			let object: &A = self.get_object(name.as_str()).unwrap();
			return Ok(object);
		} else {
			// ID isn't valid
			return Err(Error::new(format!("Default cached object with ID '{}' does not exist!", id).as_str()));
		}
	}
	// Get the ID of an object using it's name
	pub fn get_object_id(&self, name: &str) -> Result<u16, Error> {
		if self.defaults.contains(&name.to_string()) {
			// That default object name is valid
			let id = self.names[name].clone();
			return Ok(id);
		} else {
			// Name's not valid
			return Err(Error::new(format!("Object name '{}' is invalid!", name).as_str()));
		}
	}
	// Cached an object and gives back it's cached ID
	pub fn cache_object(&mut self, object: A, name: &str) -> u16 {
		// Cache the object, if it was already cached then just return it's id
		if self.names.contains_key(&name.to_string()) {
			let id = self.names.get(&name.to_string()).unwrap();
			return id.clone();
		} else {
			// The object was never cached, so we've gotta cache it
			self.objects.push(object);
			self.names.insert(name.to_string(), self.objects.len() as u16 - 1);
			let id = self.objects.len() as u16 - 1;
			return id;
		}
	} 	
	// Get a reference to an object using it's object name
	pub fn get_object(&self, name: &str) -> Option<&A> {
		if self.names.contains_key(name) {
			// The object exists, we can safely return it
			return Some(self.objects.get(self.names[&name.to_string()] as usize).unwrap());
		} else {
			// The object does not exist, return none 
			return None;
		}
	}
	// Get a reference to an object using it's object name
	pub fn get_object_mut(&mut self, name: &str) -> Option<&mut A> {
		if self.names.contains_key(name) {
			// The object exists, we can safely return it
			return Some(self.objects.get_mut(self.names[&name.to_string()] as usize).unwrap());
		} else {
			// The object does not exist, return none 
			return None;
		}
	}
	// Get a reference to an object using it's object ID
	pub fn id_get_object(&self, id: u16) -> &A {
		return self.objects.get(id as usize).unwrap();
	}
	// Get a reference to an object using it's object ID
	pub fn id_get_object_mut(&mut self, id: u16) -> &mut A {
		return self.objects.get_mut(id as usize).unwrap();
	}
}

// The "From Resource" trait
trait FromResource {
	fn from_resource() -> Self;
}