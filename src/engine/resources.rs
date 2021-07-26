use std::{collections::HashMap, fs::{File, read_dir}, io::{BufReader, BufWriter, Read, Write}, path::PathBuf};
use crate::engine::rendering::SubShaderType;

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
			let bytes_read = reader.read_to_end(&mut bytes).unwrap();

			// Temp variable
			let mut loaded_resource = Resource::None;

			match bytes[0] {
				_ => {}
				1 => {
					// This is a model
				}
				2 => {
					// This is a texture
				}
				3 => {
					// Temp shader type
					let mut shader_type = SubShaderType::Fragment;
					// This is a shader
					match bytes[1] {
						0 => shader_type = SubShaderType::Vertex,
						1 => shader_type = SubShaderType::Fragment,
						2 => shader_type = SubShaderType::Geometry,
						_ => {	}
					}
					// This is a vertex subshader
					let shader_source = String::from_utf8(bytes[3..].to_vec()).unwrap();
					loaded_resource = Resource::Shader( LoadedSubShader {
						source: shader_source,
						shader_type 
					} );
				}
				4 => {
					// This is a sound effect
				}
			}
			
			// Cache the resource so we can use it later without the need to reload
			println!("Cache resource {}", name);
			self.cached_resources.insert(name.clone(), loaded_resource);
			Some(self.cached_resources.get(&name).unwrap())
		}
	}
	// Unloads a resource to save on memory
	pub fn unload_resouce() {

	}
	// Saves all the resources from the "resources" folder into the "packed-resources" folder
	pub fn save_resources() {
		// Get all the resource files from the resources folder
		let files = read_dir("../resources/").expect("Failed to read the resources directory!");
		let packaged_resource_path = "../packaged-resources/";

		// Now, pack every resource
		for file in files {
			let path = file.unwrap().path();
			let extension = path.extension().unwrap();
			let name = path.file_name().unwrap();
			let opened_file = File::open(&path).unwrap();
			let mut reader = BufReader::new(opened_file);
			let mut bytes: Vec<u8> = Vec::new();
			let mut resource: Resource = Resource::None;
			// Read the file bytes into the vector
			reader.read_exact(&mut bytes);

			match extension.to_str().unwrap() {
				"vrtsh" => {
					// This is a vertex shader
					let mut string_source: String = String::new();
					reader.read_to_string(&mut string_source);
					resource = Resource::Shader(LoadedSubShader { source: string_source, shader_type: SubShaderType::Vertex });					
				}
				"frtsh" => {
					// This is a fragment shader
					let mut string_source: String = String::new();
					reader.read_to_string(&mut string_source);
					resource = Resource::Shader(LoadedSubShader { source: string_source, shader_type: SubShaderType::Fragment });
				}
				"png" => {
					// This is a texture
				}
				"wav" => {
					// This is a sound effect
				}
				"obj" => {
					// This is a model
				}
				_ => println!("File type not supported!")
			}
			
			// Create the packaged file
			let new_file = File::create(format!("{}/{}", &packaged_resource_path, name.to_str().unwrap())).expect("failed to create the packaged file!");
			// These are the bytes that we are going to write to the file
			let mut bytes_to_write: Vec<u8> = Vec::new();
			
			// The first byte is the type of resource that we will be loading in
			let mut resource_type: u8 = 0;

			// Now we can serialize each type of resource and pack them
			match resource {
				Resource::None => resource_type = 0,
    			Resource::Model(_) => {
					resource_type = 1; 
				},
    			Resource::Texture(_) => {
					resource_type = 2;
				},
    			Resource::Shader(shader) => {
					// Turn the source string into bytes, and write them into the resource file
					resource_type = 3;
					let mut string_bytes = shader.source.into_bytes().to_vec();
					let mut shader_type_byte: u8 = 0;
					// Save the type of subshader
					match shader.shader_type {
        				SubShaderType::Vertex => shader_type_byte = 0,
        				SubShaderType::Fragment => shader_type_byte = 1,
        				SubShaderType::Geometry => shader_type_byte = 2,
					}
					bytes_to_write.append(&mut vec![shader_type_byte]);

					// Save the shader source code found in the text files
					bytes_to_write.append(&mut string_bytes);
				},
    			Resource::Sound(_) => {
					resource_type = 4
				},
			}

			// Now we can actually write the bytes
			bytes_to_write.append(&mut vec![resource_type]);
			let mut writer = BufWriter::new(new_file);
			writer.write(bytes_to_write.as_slice());
		}
	}
}

// A simple loaded resource
pub enum Resource {
	None,
	Model(LoadedModel),
	Texture(LoadedTexture),
	Shader(LoadedSubShader),
	Sound(LoadedSoundEffect),
}

// Default enum for ResourceType
impl Default for Resource {
	fn default() -> Self {
		Self::None
	}
}

// A loaded model resource
pub struct LoadedModel {
	pub vertices: Vec<(f32, f32, f32)>,
	pub triangles: Vec<u32>,
}
// A loaded texture resource
pub struct LoadedTexture {
	pub raw_pixels: Vec<(u8, u8, u8)>
}
// A loaded sub shader
pub struct LoadedSubShader {
	pub source: String,
	pub shader_type: SubShaderType,
}
// A sound effect that can be played at any time
pub struct LoadedSoundEffect {	}