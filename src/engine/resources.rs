use std::{collections::HashMap, fs::File, io::{BufReader, Read}};

// A resource manager that will load structs from binary files
#[derive(Default)]
pub struct ResourceManager {
	cached_resources: HashMap<String, Resource>,
}

// Da code
impl ResourceManager {
	// Loads a specific resource and caches it so we can use it next time
	pub fn load_resource(&mut self, name: String, path: String) -> Option<&Resource> {
		// First of all, check if we have that resource cached
		if self.cached_resources.contains_key(&name) {
			// Return the cached resource
			println!("Load cached resource {} from path {}", name, path);
			Some(self.cached_resources.get(&name).unwrap())
		} else {
			// If not, load a new resource
			let file = File::open(format!("{}{}", path, name)).expect("The resource file did not load properly!");
			let mut reader = BufReader::new(file);

			// The bytes that will be turned into the resource
			let mut bytes: Vec<u8> = Vec::new();
			// Read each byte
			let byte_read = reader.read_to_end(&mut bytes);

			// Blah blah blah..., loading resource done
			let loaded_resource = Resource { data: ResourceType::None };

			// Cache the resource so we can use it later without the need to reload
			println!("Cache resource {}", name);
			self.cached_resources.insert(name.clone(), loaded_resource);
			Some(self.cached_resources.get(&name).unwrap())
		}
	}
}

// The resource type
#[derive(Clone)]
pub enum ResourceType {
	Model(LoadedModel),
	Texture(LoadedTexture),
	Shader(LoadedSubShader),
	Sound(LoadedSoundEffect),
	None
}

// Default enum for ResourceType
impl Default for ResourceType {
	fn default() -> Self {
		Self::None
	}
}

// A loaded model resource
#[derive(Clone)]
pub struct LoadedModel {
	pub vertices: Vec<(f32, f32, f32)>,
	pub triangles: Vec<u32>,
}
// A loaded texture resource
#[derive(Clone)]
pub struct LoadedTexture {
	pub raw_pixels: Vec<(u8, u8, u8)>
}
// A loaded sub shader
#[derive(Clone)]
pub struct LoadedSubShader {
	pub source: String,
	pub shader_type: u8,
}
// A sound effect that can be played at any time
#[derive(Clone)]
pub struct LoadedSoundEffect {	}

// A simple resource, it will just point to a space in memory that will either contain a specific resource type
#[derive(Default, Clone)]
struct Resource {
	pub data: ResourceType,
}