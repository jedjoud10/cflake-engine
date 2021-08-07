use std::{collections::HashMap, env, ffi::OsStr, fs::{File, create_dir, create_dir_all, read_dir}, io::{BufRead, BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write}, path::{Path, PathBuf}, str, thread::current};
use crate::engine::rendering::SubShaderType;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use image::{GenericImageView, io::Reader as ImageReader};

// A resource manager that will load structs from binary files
#[derive(Default)]
pub struct ResourceManager {
	cached_resources: HashMap<String, Resource>,
}

// Da code
impl ResourceManager {
	// Loads a specific resource and caches it so we can use it next time
	pub fn load_resource(&mut self, name: &str, path: &str) -> Option<&Resource> {
		let name = String::from(name);
		let path = String::from(path);
		let mut final_path: String = String::new();
		{
			let dir_original = env::current_exe().unwrap();
			let dir_split_temp = dir_original.to_str().unwrap().split("\\");
			let dir_split_len = dir_split_temp.clone().count();
			let exe_dir = dir_split_temp
				.enumerate()
				.filter(|&(i, _)| i < dir_split_len - 1)
				.map(|(_i, s)| s.to_string())
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
			let file = File::open(file_path).expect(format!("The resource file '{}' could not be read!", &name).as_str());
			let mut reader = BufReader::new(file);

			// The bytes that will be turned into the resource
			let mut bytes: Vec<u8> = Vec::new();
			// Read each byte
			let _bytes_read = reader.read_to_end(&mut bytes).unwrap();
			// Temp variable
			reader.seek(SeekFrom::Start(0));
			let mut loaded_resource = Resource::None;

			match reader.read_u8().unwrap() {
				1 => {
					// This is a model
					let vertices_size: u32 = reader.read_u32::<LittleEndian>().unwrap();
					let triangles_size: u32 = reader.read_u32::<LittleEndian>().unwrap();
					let mut vertices: Vec<glam::Vec3> = Vec::new();
					let mut triangles: Vec<u32> = Vec::new();
					let mut normals: Vec<glam::Vec3> = Vec::new();
					let mut uvs: Vec<glam::Vec2> = Vec::new();
					let mut tangents: Vec<glam::Vec3> = Vec::new();
					// Load the vertices
					for _i in 0..vertices_size {
						vertices.push(glam::vec3(reader.read_f32::<LittleEndian>().unwrap(), reader.read_f32::<LittleEndian>().unwrap(), reader.read_f32::<LittleEndian>().unwrap()));
					}
					// Load the normals
					for _i in 0..vertices_size {
						normals.push(glam::vec3(reader.read_f32::<LittleEndian>().unwrap(), reader.read_f32::<LittleEndian>().unwrap(), reader.read_f32::<LittleEndian>().unwrap()));
					}
					// Load the tangents
					for _i in 0..vertices_size {
						tangents.push(glam::vec3(reader.read_f32::<LittleEndian>().unwrap(), reader.read_f32::<LittleEndian>().unwrap(), reader.read_f32::<LittleEndian>().unwrap()));
					}
					// Load the uvs
					for _i in 0..vertices_size {
						uvs.push(glam::vec2(reader.read_f32::<LittleEndian>().unwrap(), reader.read_f32::<LittleEndian>().unwrap()));
					}

					// Load the triangles
					for _i in 0..triangles_size {
						triangles.push(reader.read_u32::<LittleEndian>().unwrap());
					}
					// Convert the bytes into a loaded model
					loaded_resource = Resource::Model(LoadedModel {
						vertices,
						normals,
						tangents,
						uvs,
						indices: triangles
					});
				}
				2 => {
					// This is a texture
					let texture_width = reader.read_u16::<LittleEndian>().unwrap();
					let texture_height = reader.read_u16::<LittleEndian>().unwrap();
					let mut bytes: Vec<u8> = Vec::new();
					for x in 0..texture_width {
						for y in 0..texture_height {
							for color in 0..3 {
								bytes.push(reader.read_u8().unwrap());
							}
						}
					}

					// Load the bytes into the resource
					loaded_resource = Resource::Texture(LoadedTexture {
						name: name.clone(),
        				width: texture_width,
        				height: texture_height,
        				raw_pixels: bytes,
    				})
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
							// This is a texture
							let image_bytes = bytes;
							let image = ImageReader::open(path)
								.unwrap()
								.with_guessed_format()
								.unwrap()
								.decode()
								.unwrap();
							let dimensions = image.dimensions();
							resource = Resource::Texture(LoadedTexture {
								name: String::from("Undefined"),
								width: dimensions.0 as u16,
								height: dimensions.1 as u16,
								raw_pixels: image.to_bytes(),
							});
							resource_type = 2;
						}
						"wav" => {
							resource_type = 4;
							// This is a sound effect
						}
						"mdl3d" => {
							resource_type = 1;
							// This is a model
							// Parse the obj model
							let mut vertices: Vec<glam::Vec3> = Vec::new(); 
							let mut normals: Vec<glam::Vec3> = Vec::new(); 
							let mut tangents: Vec<glam::Vec3> = Vec::new(); 
							let mut uvs: Vec<glam::Vec2> = Vec::new(); 
							let mut triangles: Vec<u32> = Vec::new();
							for line in reader.lines() {
								let line = line.unwrap();
								let start = line.split_once(" ").unwrap().0;
								let other = line.split_once(" ").unwrap().1;
								match start {
									"v" => {
										let coords: Vec<f32> = other.split("/").map(|coord| coord.parse::<f32>().unwrap()).collect();
										vertices.push(glam::vec3(coords[0], coords[1], coords[2]));
									}
									"n" => {
										let coords: Vec<f32> = other.split("/").map(|coord| coord.parse::<f32>().unwrap()).collect();
										normals.push(glam::vec3(coords[0], coords[1], coords[2]));
									}
									"u" => {
										let coords: Vec<f32> = other.split("/").map(|coord| coord.parse::<f32>().unwrap()).collect();
										uvs.push(glam::vec2(coords[0], coords[1]));
									}
									"t" => {
										let coords: Vec<f32> = other.split("/").map(|coord| coord.parse::<f32>().unwrap()).collect();
										tangents.push(glam::vec3(coords[0], coords[1], coords[2]));
									}
									// Load the triangle indices
									"i" => {
										// Split the triangle into 3 indices
										let mut indices = other.split("/").map(|x| x.to_string().parse::<u32>().unwrap()).collect();
										triangles.append(&mut indices);
									}
									_ => {	}
								}
							}
							resource = Resource::Model(LoadedModel {
    							vertices,
    							indices: triangles,
								normals,
								uvs,
								tangents,
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
							writer.write_u32::<LittleEndian>(model.vertices.len() as u32);
							writer.write_u32::<LittleEndian>(model.indices.len() as u32);
							// Write the vertices
							for &vertex in model.vertices.iter() {
								writer.write_f32::<LittleEndian>(vertex.x);
								writer.write_f32::<LittleEndian>(vertex.y);
								writer.write_f32::<LittleEndian>(vertex.z);
							}
							// Write the normals
							for &normal in model.normals.iter() {
								writer.write_f32::<LittleEndian>(normal.x);
								writer.write_f32::<LittleEndian>(normal.y);
								writer.write_f32::<LittleEndian>(normal.z);
							}
							// Write the tangents
							for &tangent in model.tangents.iter() {
								writer.write_f32::<LittleEndian>(tangent.x);
								writer.write_f32::<LittleEndian>(tangent.y);
								writer.write_f32::<LittleEndian>(tangent.z);
							}
							// Write the uvs
							for &uv in model.uvs.iter() {
								writer.write_f32::<LittleEndian>(uv.x);
								writer.write_f32::<LittleEndian>(uv.y);
							}
							// Write the indices
							for &index in model.indices.iter() {
								writer.write_u32::<LittleEndian>(index);
							}
						},
    					Resource::Texture(texture) => {
							// Write the dimensions of the texture
							writer.write_u16::<LittleEndian>(texture.width);
							writer.write_u16::<LittleEndian>(texture.height);

							// Now write all the bytes
							for byte in texture.raw_pixels {
								writer.write_u8(byte);
							}
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
#[derive(Debug)]
pub struct LoadedModel {
	pub vertices: Vec<glam::Vec3>,
	pub normals: Vec<glam::Vec3>,
	pub tangents: Vec<glam::Vec3>,
	pub uvs: Vec<glam::Vec2>,
	pub indices: Vec<u32>,
}
// A loaded texture resource
pub struct LoadedTexture {
	pub name: String,
	pub width: u16,
	pub height: u16,
	pub raw_pixels: Vec<u8>
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