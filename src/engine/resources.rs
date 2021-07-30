use std::{collections::HashMap, env, ffi::OsStr, fs::{File, create_dir, create_dir_all, read_dir}, io::{BufRead, BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write}, path::{Path, PathBuf}, str, thread::current};
use crate::engine::rendering::SubShaderType;
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian, LittleEndian};

// A resource manager that will load structs from binary files
#[derive(Default)]
pub struct ResourceManager {
	cached_resources: HashMap<String, Resource>,
}

// Da code
impl ResourceManager {
	// Loads a specific resource and caches it so we can use it next time
	pub fn load_resource(&mut self, name: String, path: String) -> Option<&Resource> {
		let mut final_path: String = String::new();
		{
			let dir_original = env::current_exe().unwrap();
			let dir_split_temp = dir_original.to_str().unwrap().split("\\");
			let dir_split_len = dir_split_temp.clone().count();
			let exe_dir = dir_split_temp
				.enumerate()
				.filter(|&(i, _)| i < dir_split_len - 1)
				.map(|(i, s)| s.to_string())
				.into_iter()
				.collect::<Vec<String>>()
				.join(char::to_string(&'\\').as_str());
			final_path = format!("{}\\packed-resources\\{}", exe_dir.as_str(), path);	
		}
		// First of all, check if we have that resource cached
		if self.cached_resources.contains_key(&name) {
			// Return the cached resource
			println!("Load cached resource '{}' from path {}", name, final_path);
			return Some(self.cached_resources.get(&name).unwrap());
		} else {
			// If not, load a new resource
			let file_path = format!("{}{}", final_path, name);
			let file = File::open(file_path).expect("The resource file did not load properly!");
			let mut reader = BufReader::new(file);

			// The bytes that will be turned into the resource
			let mut bytes: Vec<u8> = Vec::new();
			// Read each byte
			let bytes_read = reader.read_to_end(&mut bytes).unwrap();
			// Temp variable
			reader.seek(SeekFrom::Start(0));
			let mut loaded_resource = Resource::None;

			match reader.read_u8().unwrap() {
				1 => {
					// This is a model
					let vertices_size: u16 = reader.read_u16::<LittleEndian>().unwrap();
					let triangles_size: u16 = reader.read_u16::<LittleEndian>().unwrap();
					let mut vertices: Vec<glam::Vec3> = Vec::new();
					let mut triangles: Vec<u32> = Vec::new();
					for i in 0..vertices_size {
						vertices.push(glam::vec3(reader.read_f32::<LittleEndian>().unwrap(), reader.read_f32::<LittleEndian>().unwrap(), reader.read_f32::<LittleEndian>().unwrap()));
					}
					for i in 0..triangles_size {
						triangles.push(reader.read_u32::<LittleEndian>().unwrap());
					}
					// Convert the bytes into a loaded model
					loaded_resource = Resource::Model(LoadedModel {
						vertices: vertices,
						indices: triangles
					});
				}
				2 => {
					// This is a texture
				}
				3 => {
					// Temp shader type
					let mut shader_type = SubShaderType::Fragment;
					let mut shader_name: String = String::new();
					// This is a shader
					match bytes[1] {
						0 => {
							shader_type = SubShaderType::Vertex;
							shader_name = format!("{}.{}", &name, "vertex");
						},
						1 => { 
							shader_type = SubShaderType::Fragment;
							shader_name = format!("{}.{}", &name, "fragment");
						},
						2 => { 
							shader_type = SubShaderType::Geometry;
							shader_name = format!("{}.{}", &name, "geometry");
						},
						_ => {	}
					}
					// This is a vertex subshader
					let shader_source = String::from_utf8(bytes[2..].to_vec()).unwrap();
					loaded_resource = Resource::Shader(LoadedSubShader {
						name: shader_name,
						source: shader_source.clone(),
						subshader_type: shader_type.clone(),
					});
				}
				4 => {
					// This is a sound effect
				}
				_ => {}
			}
			
			// Cache the resource so we can use it later without the need to reload
			println!("Cache resource: '{}'", name);
			self.cached_resources.insert(name.clone(), loaded_resource);
			return Some(self.cached_resources.get(&name).unwrap());
		}
	}
	// Unloads a resource to save on memory
	pub fn unload_resouce(&mut self) {

	}
	// Saves all the resources from the "resources" folder into the "packed-resources" folder
	pub fn pack_resources() {
		println!("Packing resources...");
		// Go up by two folders, so we're in the main project folder
		let path = env::current_dir().unwrap().clone();
		let path = path
			.parent()
			.unwrap()
			.parent()
			.unwrap();
		let path = path.as_os_str().to_str().unwrap();
		let resources_dir = format!("{}\\src\\resources\\", path);
		let packed_resources_dir = format!("{}\\src\\packed-resources\\", path);
		println!("Resources directory: {}", &resources_dir);
		println!("Packed-Resources directory: {}", &packed_resources_dir);
		// Get all the resource files from the resources folder
		let files = read_dir(resources_dir).expect("Failed to read the resources directory!");
		// Now, pack every resource in each sub-directory
		for sub_directory in files {
			let sub_directory = sub_directory.as_ref().unwrap();
			let md = &sub_directory.metadata().unwrap();
			// This is a folder, so save the folder into the packed resources as well
			if md.is_dir() {
				let sub_dir_files = read_dir(sub_directory.path().to_str().unwrap()).expect("Failed to read a resources sub-directory!");				
				for sub_file in sub_dir_files {		
					let sub_file = sub_file.as_ref().unwrap();		
					let path = sub_file.path();
					// This extension is anything after the first dot
					let extension: Vec<&str> = path.file_name().unwrap().to_str().unwrap().split(".").collect();
					let extension = &extension[1..].join(".");
					let name: String = path.file_name().unwrap().to_str().unwrap().split(".").nth(0).unwrap().to_string();
					let sub_dir_name = sub_directory.file_name();
					println!("Directory name: '{}'", sub_dir_name.to_str().unwrap());
					let packed_resources_dir = format!("{}{}", packed_resources_dir, sub_dir_name.to_str().unwrap());
					println!("Packing resource: '{}'", name.as_str());
					let opened_file = File::open(&path).unwrap();
					let mut reader = BufReader::new(opened_file);
					let mut bytes: Vec<u8> = Vec::new();
					let mut resource: Resource = Resource::None;
					// Read the file bytes into the vector
					reader.read_exact(&mut bytes);

					// The type of resource that we will be saving
					let mut resource_type = 0;
					
					match extension.as_str() {
						"vrsh.glsl" => {
							// This is a vertex shader
							let mut string_source: String = String::new();
							reader.read_to_string(&mut string_source);
							resource = Resource::Shader(LoadedSubShader { name: String::from("Undefined"), source: string_source, subshader_type: SubShaderType::Vertex });	
							resource_type = 3;				
						}
						"frsh.glsl" => {
							// This is a fragment shader
							let mut string_source: String = String::new();
							reader.read_to_string(&mut string_source);
							resource = Resource::Shader(LoadedSubShader { name: String::from("Undefined"), source: string_source, subshader_type: SubShaderType::Fragment });
							resource_type = 3;
						}
						"png" => {
							resource_type = 2;
							// This is a texture
						}
						"wav" => {
							resource_type = 4;
							// This is a sound effect
						}
						"obj" => {
							resource_type = 1;
							// This is a model
							// Parse the obj model
							let mut vertices: Vec<glam::Vec3> = Vec::new(); 
							let mut triangles: Vec<u32> = Vec::new();
							for line in reader.lines() {
								let line = line.unwrap();
								let start = line.split_once(" ").unwrap().0;
								let other = line.split_once(" ").unwrap().1;
								match start {
									"v" => {
										let coords: Vec<f32> = other.split(" ").map(|coord| coord.parse::<f32>().unwrap()).collect();
										vertices.push(glam::vec3(coords[0], coords[1], coords[2]));
									}
									"f" => {
										// Get only the index part of the main triangle
										let triangle_string: Vec<String> = other.split(" ").map(|x| x.to_string()).collect();
										let mut indices: Vec<u32> = Vec::new();
										// Repeats 3 times, for each triangle in the model
										for data_strip in triangle_string.iter() {
											let first_slash = data_strip.find("/").unwrap();
											// Use the first slash's location to find the second one
											let second_slash = data_strip[first_slash..].find("/").unwrap();
											let index = data_strip[..first_slash].to_string().parse::<u32>().unwrap();
											indices.push(index-1);
										}

										triangles.append(&mut indices);
									}
									_ => {	}
								}
							}
							resource = Resource::Model(LoadedModel {
    							vertices: vertices,
    							indices: triangles,
							});				
						}
						_ => { 
							println!("File type not supported!");
							continue;
						}
					}
					
					// Now we actually write to the file
					
					// Make sure the packed resources directory exists
					let temp_path = format!("{}\\", &packed_resources_dir);
					let temp_path = Path::new(&temp_path);
					if !temp_path.exists() {
						create_dir_all(&temp_path);
					}
					
					let packed_file_path = format!("{}\\{}.{}.pkg", &packed_resources_dir, name.as_str(), extension.as_str());
					println!("{}", packed_file_path);
					// Create the new file
					let new_file = File::create(packed_file_path).expect("Failed to create the packaged file!");
					// These are the bytes that we are going to write to the file
					let mut bytes_to_write: Vec<u8> = Vec::new();								
					// Write the resource type first
					let mut writer = BufWriter::new(new_file);
					writer.write_u8(resource_type);
					
					// Now we can serialize each type of resource and pack them
					match resource {
						Resource::None => {	},
    					Resource::Model(model) => {
							// Write to the strem
							writer.write_u16::<LittleEndian>(model.vertices.len() as u16);
							writer.write_u16::<LittleEndian>(model.indices.len() as u16);
							for &vertex in model.vertices.iter() {
								writer.write_f32::<LittleEndian>(vertex.x);
								writer.write_f32::<LittleEndian>(vertex.y);
								writer.write_f32::<LittleEndian>(vertex.z);
							}
							for &index in model.indices.iter() {
								writer.write_u32::<LittleEndian>(index);
							}
						},
    					Resource::Texture(_) => {
						},
    					Resource::Shader(shader) => {
							// Turn the source string into bytes, and write them into the resource file
							let mut string_bytes = shader.source.into_bytes().to_vec();
							let mut shader_type_byte: u8 = 0;
							// Save the type of subshader
							match shader.subshader_type {
								SubShaderType::Vertex => shader_type_byte = 0,
        						SubShaderType::Fragment => shader_type_byte = 1,
        						SubShaderType::Geometry => shader_type_byte = 2,
							}
							bytes_to_write.append(&mut vec![shader_type_byte]);
							
							// Save the shader source code found in the text files
							bytes_to_write.append(&mut string_bytes);
						},
    					Resource::Sound(_) => {
						},
					}
					
					// Create the packaged file
					//panic!("{}", packed_file_path);
					writer.write(bytes_to_write.as_slice());
				}
			}
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
	pub vertices: Vec<glam::Vec3>,
	pub indices: Vec<u32>,
}
// A loaded texture resource
pub struct LoadedTexture {
	pub raw_pixels: Vec<(u8, u8, u8)>
}
// A loaded sub shader
#[derive(Clone)]
pub struct LoadedSubShader {
	pub name: String,
	pub source: String,
	pub subshader_type: SubShaderType,
}
// A sound effect that can be played at any time
pub struct LoadedSoundEffect {	}