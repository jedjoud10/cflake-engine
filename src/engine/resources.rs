use crate::engine::rendering::shader::SubShaderType;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use image::{io::Reader as ImageReader, GenericImageView};
use std::{
	collections::{hash_map::DefaultHasher, HashMap},
	env,
	ffi::OsStr,
	fs::{create_dir, create_dir_all, read_dir, remove_file, File, OpenOptions},
	hash::{Hash, Hasher},
	io::{BufRead, BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write},
	os::windows::prelude::MetadataExt,
	path::{Path, PathBuf},
	str,
	thread::current,
	time::SystemTime,
};

// A resource manager that will load structs from binary files
#[derive(Default)]
pub struct ResourceManager {
	cached_resources: HashMap<String, Resource>,
}

// Da code.
// Date: 2021-08-08. Warning: Do not touch this code. It will give you headaches. Trust me.
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
			let file = File::open(file_path)
				.expect(format!("The resource file '{}' could not be read!", &name).as_str());
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
					let mut tangents: Vec<glam::Vec4> = Vec::new();
					// Load the vertices
					for _i in 0..vertices_size {
						vertices.push(glam::vec3(
							reader.read_f32::<LittleEndian>().unwrap(),
							reader.read_f32::<LittleEndian>().unwrap(),
							reader.read_f32::<LittleEndian>().unwrap(),
						));
					}
					// Load the normals
					for _i in 0..vertices_size {
						normals.push(glam::vec3(
							reader.read_f32::<LittleEndian>().unwrap(),
							reader.read_f32::<LittleEndian>().unwrap(),
							reader.read_f32::<LittleEndian>().unwrap(),
						));
					}
					// Load the tangents
					for _i in 0..vertices_size {
						tangents.push(glam::vec4(
							reader.read_f32::<LittleEndian>().unwrap(),
							reader.read_f32::<LittleEndian>().unwrap(),
							reader.read_f32::<LittleEndian>().unwrap(),
							reader.read_f32::<LittleEndian>().unwrap(),
						));
					}
					// Load the uvs
					for _i in 0..vertices_size {
						uvs.push(glam::vec2(
							reader.read_f32::<LittleEndian>().unwrap(),
							reader.read_f32::<LittleEndian>().unwrap(),
						));
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
						indices: triangles,
					});
				}
				2 => {
					// This is a texture
					let texture_width = reader.read_u16::<LittleEndian>().unwrap();
					let texture_height = reader.read_u16::<LittleEndian>().unwrap();
					let mut compressed_bytes: Vec<u8> = Vec::new();
					reader.seek(SeekFrom::Start(5)).unwrap();
					reader.read_to_end(&mut compressed_bytes);

					// Load the bytes into the resource
					loaded_resource = Resource::Texture(LoadedTexture {
						name: name.clone(),
						width: texture_width,
						height: texture_height,
						compressed_bytes,
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
						}
						1 => {
							shader_type = SubShaderType::Fragment;
							shader_name = format!("{}.{}", &name, "fragment");
						}
						2 => {
							shader_type = SubShaderType::Geometry;
							shader_name = format!("{}.{}", &name, "geometry");
						}
						_ => {}
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
	pub fn unload_resouce(&mut self) {}
	// Saves all the resources from the "resources" folder into the "packed-resources" folder
	pub fn pack_resources() {
		println!("Packing resources...");
		// Go up by two folders, so we're in the main project folder
		let path = env::current_dir().unwrap().clone();
		let path = path.parent().unwrap().parent().unwrap();
		let path = path.as_os_str().to_str().unwrap();
		let resources_dir = format!("{}\\src\\resources\\", path);
		let packed_resources_dir = format!("{}\\src\\packed-resources\\", path);
		println!("Resources directory: {}", &resources_dir);
		println!("Packed-Resources directory: {}", &packed_resources_dir);
		// Get a writer to the log file
		// Keep track of the names-timestamp relation
		let log_file_path = format!("{}\\src\\packed-resources\\log.log", path);
		let mut hashed_names_timestamps: HashMap<u64, u64> = HashMap::new();
		{
			let mut log_reader = BufReader::new(
				OpenOptions::new()
					.write(true)
					.read(true)
					.create(true)
					.open(log_file_path.clone())
					.unwrap(),
			);
			let mut num = 0;
			let mut last_hashed_name = 0_u64;
			// Make an infinite loop, and at each iteration, read 8 bytes
			// Those 8 bytes will either be a hashed-name that we will store for the next iteration or
			// A timestamp, if it's a timestamp then get the hashed-name we got from the last iteration and insert both of them into the hashmap
			loop {
				match log_reader.read_u64::<LittleEndian>() {
					Ok(val) => {
						// Check if this is a hashed name or a timestamp
						if num % 2 == 0 {
							// This is a hashed name
							last_hashed_name = val;
						} else {
							// This is a timestamp
							hashed_names_timestamps.insert(last_hashed_name.clone(), val);
						}
						num += 1;
					}
					Err(_) => {
						// End of the log file
						break;
					}
				}
			}
		}
		println!("{:?}", hashed_names_timestamps);
		// Get all the resource files from the resources folder
		let files =
			read_dir(resources_dir.clone()).expect("Failed to read the resources directory!");
		let mut files_that_could_possibly_get_packed: Vec<String> = Vec::new();
		// Now, pack every resource in each sub-directory
		for sub_directory in files {
			let sub_directory = sub_directory.as_ref().unwrap();
			let md = &sub_directory.metadata().unwrap();
			// This is a folder, so save the folder into the packed resources as well
			if md.is_dir() {
				let sub_dir_files = read_dir(sub_directory.path().to_str().unwrap())
					.expect("Failed to read a resources sub-directory!");
				for sub_file in sub_dir_files {
					let sub_file = sub_file.as_ref().unwrap();
					let path = sub_file.path();
					// This extension is anything after the first dot
					let extension: Vec<&str> = path
						.file_name()
						.unwrap()
						.to_str()
						.unwrap()
						.split(".")
						.collect();
					let extension = &extension[1..].join(".");
					let name: String = path
						.file_name()
						.unwrap()
						.to_str()
						.unwrap()
						.split(".")
						.nth(0)
						.unwrap()
						.to_string();
					let sub_dir_name = sub_directory.file_name();
					println!("Directory name: '{}'", sub_dir_name.to_str().unwrap());
					let packed_resources_dir =
						format!("{}{}", packed_resources_dir, sub_dir_name.to_str().unwrap());
					println!("Packing resource: '{}'", name.as_str());
					let opened_file = File::open(&path).unwrap();
					files_that_could_possibly_get_packed.push(format!(
						"{}\\{}",
						sub_dir_name.to_str().unwrap(),
						name
					));
					// Check the logged timestamp of this resource, if it does not exist, then pack this resource
					{
						// Hash the name
						let mut hasher = DefaultHasher::new();
						format!("{}\\{}", sub_dir_name.to_str().unwrap(), name).hash(&mut hasher);
						let hashed_name = hasher.finish();
						// The timestamp of the current resource
						let resource_timestamp = opened_file
							.metadata()
							.unwrap()
							.modified()
							.unwrap()
							.duration_since(std::time::UNIX_EPOCH)
							.unwrap()
							.as_secs();
						if hashed_names_timestamps.contains_key(&hashed_name) {
							// Check if the timestamp changed drastically (Margin of 20 seconds)
							let packed_resource_timestamp =
								hashed_names_timestamps.get(&hashed_name).unwrap().clone();
							// Did we update the resource?
							if resource_timestamp > packed_resource_timestamp {
								// We did update
								println!("Resource file did change!");
							} else {
								// We did not update, no need to pack this resource
								println!("Resource file did not change...");
								continue;
							}
						} else {
							// The resource just got added, so we pack it
							println!("Could not find packed-resource data in log file...");
						}
					}

					// Write the extension to the file
					let mut reader = BufReader::new(opened_file);
					let mut bytes: Vec<u8> = Vec::new();
					let mut resource: Resource = Resource::None;
					// Read the file bytes into the vector
					let bytes_read = reader.read_to_end(&mut bytes).unwrap();
					reader.seek(SeekFrom::Start(0));
					println!("Bytes read: {}", bytes_read);

					// The type of resource that we will be saving
					let mut resource_type = 0;

					match extension.as_str() {
						"vrsh.glsl" => {
							// This is a vertex shader
							let mut string_source: String = String::new();
							reader.read_to_string(&mut string_source);
							resource = Resource::Shader(LoadedSubShader {
								name: String::from("Undefined"),
								source: string_source,
								subshader_type: SubShaderType::Vertex,
							});
							resource_type = 3;
						}
						"frsh.glsl" => {
							// This is a fragment shader
							let mut string_source: String = String::new();
							reader.read_to_string(&mut string_source);
							resource = Resource::Shader(LoadedSubShader {
								name: String::from("Undefined"),
								source: string_source,
								subshader_type: SubShaderType::Fragment,
							});
							resource_type = 3;
						}
						"png" => {
							// This is a texture
							let image = ImageReader::open(path.clone()).unwrap().with_guessed_format().unwrap().decode().unwrap(); 
							let dimensions = image.dimensions();
							println!("{:?}", dimensions);
							resource = Resource::Texture(LoadedTexture {
								name: String::from("Undefined"),
								width: dimensions.0 as u16,
								height: dimensions.1 as u16,
								compressed_bytes: bytes,
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
							let mut tangents: Vec<glam::Vec4> = Vec::new();
							let mut uvs: Vec<glam::Vec2> = Vec::new();
							let mut triangles: Vec<u32> = Vec::new();
							for line in reader.lines() {
								let line = line.unwrap();
								let start = line.split_once(" ").unwrap().0;
								let other = line.split_once(" ").unwrap().1;
								match start {
									"v" => {
										let coords: Vec<f32> = other
											.split("/")
											.map(|coord| coord.parse::<f32>().unwrap())
											.collect();
										vertices.push(glam::vec3(coords[0], coords[1], coords[2]));
									}
									"n" => {
										let coords: Vec<f32> = other
											.split("/")
											.map(|coord| coord.parse::<f32>().unwrap())
											.collect();
										normals.push(glam::vec3(coords[0], coords[1], coords[2]));
									}
									"u" => {
										let coords: Vec<f32> = other
											.split("/")
											.map(|coord| coord.parse::<f32>().unwrap())
											.collect();
										uvs.push(glam::vec2(coords[0], coords[1]));
									}
									"t" => {
										let coords: Vec<f32> = other
											.split("/")
											.map(|coord| coord.parse::<f32>().unwrap())
											.collect();
										tangents.push(glam::vec4(
											coords[0], coords[1], coords[2], coords[3],
										));
									}
									// Load the triangle indices
									"i" => {
										// Split the triangle into 3 indices
										let mut indices = other
											.split("/")
											.map(|x| x.to_string().parse::<u32>().unwrap())
											.collect();
										triangles.append(&mut indices);
									}
									_ => {}
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

					let packed_file_path = format!(
						"{}\\{}.{}.pkg",
						&packed_resources_dir,
						name.as_str(),
						extension.as_str()
					);
					println!("{}", packed_file_path);
					// Create the new file
					let new_file = File::create(packed_file_path.clone())
						.expect("Failed to create the packaged file!");
					// These are the bytes that we are going to write to the file
					let mut bytes_to_write: Vec<u8> = Vec::new();
					let packed_resource_timestamp = new_file
						.metadata()
						.unwrap()
						.modified()
						.unwrap()
						.duration_since(SystemTime::UNIX_EPOCH)
						.unwrap()
						.as_secs();
					// Write the resource type first
					let mut writer = BufWriter::new(new_file);
					writer.write_u8(resource_type);

					// Now we can serialize each type of resource and pack them
					match resource {
						Resource::None => {}
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
								writer.write_f32::<LittleEndian>(tangent.w);
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
						}
						Resource::Texture(texture) => {
							// Write the dimensions of the texture
							writer.write_u16::<LittleEndian>(texture.width).unwrap();
							writer.write_u16::<LittleEndian>(texture.height).unwrap();

							// Now write all the bytes
							for byte in texture.compressed_bytes {
								writer.write_u8(byte);
							}
						}
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
						}
						Resource::Sound(_) => {}
					}

					// Create the packaged file
					//panic!("{}", packed_file_path);
					writer.write(bytes_to_write.as_slice());
					// Save the name and timestamp creation date of this packed resource in the log file
					{
						let log_file = OpenOptions::new()
							.append(true)
							.open(log_file_path.clone())
							.unwrap();
						let mut log_writer = BufWriter::new(log_file);
						let mut hashed_name: u64 = 0;
						{
							// Hash the name
							let mut hasher = DefaultHasher::new();
							format!("{}\\{}", sub_dir_name.to_str().unwrap(), name)
								.hash(&mut hasher);
							hashed_name = hasher.finish();
						}
						hashed_names_timestamps.insert(hashed_name, packed_resource_timestamp);
						log_writer.write_u64::<LittleEndian>(hashed_name);
						log_writer.write_u64::<LittleEndian>(packed_resource_timestamp);
					}
				}
			}
		}
		// Now loop through all the packed files and delete the ones that are not present in the log file
		let packed_files = read_dir(packed_resources_dir).unwrap();
		println!("{:?}", hashed_names_timestamps);
		for sub_directory in packed_files {
			let sub_directory = sub_directory.unwrap();
			let sub_dir_name = sub_directory.file_name();
			let sub_dir_name = sub_dir_name.to_str().unwrap();
			if sub_directory.metadata().unwrap().is_dir() {
				for packed_file_dir_entry in read_dir(sub_directory.path()).unwrap() {
					let packed_file = packed_file_dir_entry.as_ref().unwrap();
					let packed_file_path = packed_file.path();
					let packed_file_path = packed_file_path.to_str();
					let name = packed_file.file_name();
					let name = name.to_str().unwrap();
					let split_name_vec: Vec<&str> = name.split(".").collect();
					let split_name = split_name_vec[0];
					if files_that_could_possibly_get_packed
						.contains(&format!("{}\\{}", sub_dir_name, split_name))
					{
						// This file exists in the resources folder
					} else {
						// This file does not exist, so delete it
						remove_file(packed_file_path.unwrap());
					}
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
	pub tangents: Vec<glam::Vec4>,
	pub uvs: Vec<glam::Vec2>,
	pub indices: Vec<u32>,
}
// A loaded texture resource
pub struct LoadedTexture {
	pub name: String,
	pub width: u16,
	pub height: u16,
	pub compressed_bytes: Vec<u8>,
}
// A loaded sub shader
#[derive(Clone)]
pub struct LoadedSubShader {
	pub name: String,
	pub source: String,
	pub subshader_type: SubShaderType,
}
// A sound effect that can be played at any time
pub struct LoadedSoundEffect {}
