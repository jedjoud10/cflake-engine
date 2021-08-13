use crate::engine::rendering::shader::SubShaderType;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use image::GenericImageView;
use walkdir::WalkDir;
use std::{collections::{hash_map::DefaultHasher, HashMap}, env, ffi::OsStr, fmt::format, fs::{create_dir, create_dir_all, read_dir, remove_file, File, OpenOptions}, hash::{Hash, Hasher}, io::{BufRead, BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write}, os::windows::prelude::MetadataExt, path::{Path, PathBuf}, str, thread::current, time::SystemTime};

// A resource manager that will load structs from binary files
#[derive(Default)]
pub struct ResourceManager {
	cached_resources: HashMap<String, Resource>,
}

// All the conversion stuff from File -> Resource
impl ResourceManager {
	// Turn a mdl3d file into a LoadedModel resource
	pub fn pack_mdl3d(file: &File) -> Resource {
		// Create all the buffers
		let reader = BufReader::new(file);
		let mut vertices: Vec<glam::Vec3> = Vec::new();
		let mut normals: Vec<glam::Vec3> = Vec::new();
		let mut tangents: Vec<glam::Vec4> = Vec::new();
		let mut uvs: Vec<glam::Vec2> = Vec::new();
		let mut triangles: Vec<u32> = Vec::new();
		// Go over each line and scan it
		for line in reader.lines() {
			let line = line.unwrap();
			let start = line.split_once(" ").unwrap().0;
			let other = line.split_once(" ").unwrap().1;
			match start {
				// Vertices
				"v" => {
					let coords: Vec<f32> = other
						.split("/")
						.map(|coord| coord.parse::<f32>().unwrap())
						.collect();
					vertices.push(glam::vec3(coords[0], coords[1], coords[2]));
				}
				// Normals
				"n" => {
					let coords: Vec<f32> = other
						.split("/")
						.map(|coord| coord.parse::<f32>().unwrap())
						.collect();
					normals.push(glam::vec3(coords[0], coords[1], coords[2]));
				}
				// UVs
				"u" => {
					let coords: Vec<f32> = other
						.split("/")
						.map(|coord| coord.parse::<f32>().unwrap())
						.collect();
					uvs.push(glam::vec2(coords[0], coords[1]));
				}
				// Tangents
				"t" => {
					let coords: Vec<f32> = other
						.split("/")
						.map(|coord| coord.parse::<f32>().unwrap())
						.collect();
					tangents.push(glam::vec4(
						coords[0], coords[1], coords[2], coords[3],
					));
				}
				// Triangle indices
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
		// Create the model
		let model = LoadedModel {
			vertices,
			indices: triangles,
			normals,
			uvs,
			tangents,
		};
		return Resource::Model(model);
	}
	// Turn a shader file of any type (vertex, fragment, etc) to a LoadedShader resource
	pub fn pack_shader(file: &File, extension: &str) -> Resource {
		// The shader resource
		let mut shader: Resource = Resource::None;
		// String holding the extension of the file
		let mut reader = BufReader::new(file);
		match extension {
			"vrsh.glsl" => {
				// This is a vertex shader
				let mut string_source: String = String::new();
				reader.read_to_string(&mut string_source);
				shader = Resource::Shader(LoadedSubShader {
					name: String::from("Undefined"),
					source: string_source,
					subshader_type: SubShaderType::Vertex,
				});
			}
			"frsh.glsl" => {
				// This is a fragment shader
				let mut string_source: String = String::new();
				reader.read_to_string(&mut string_source);
				shader = Resource::Shader(LoadedSubShader {
					name: String::from("Undefined"),
					source: string_source,
					subshader_type: SubShaderType::Fragment,
				});
			}
			_ => {}
		}
		return shader;
	}
	// Turn a texture file to a LoadedTexture resource
	// While we're at it, make sure the texture has an alpha channel and EXACTLY a 24 bit depth
	pub fn pack_texture(file: &mut File, full_path: &str) -> Resource {		
		// The texture resource
		let mut texture: Resource = Resource::None;
		let mut dimensions: (u32, u32) = (0, 0);
		// Check if we even need to update the image
		let should_update: bool = {
			let mut reader = BufReader::new(file);
			let image = image::io::Reader::new(&mut reader).with_guessed_format().unwrap().decode().unwrap();
			match image {
				image::DynamicImage::ImageRgba8(_) => {
					// No need to do anything since we already have this texture at 24 bits per pixel
					false
				}
 				_ => {
					// Anything other than 24 bits
					true
				}
			}				
		};
		if should_update {
			// We need to make this it's own scope because we cannot have a reader and a writer at the same time
			let mut raw_pixels: Vec<u8> = Vec::new();
			{
				let mut reader = BufReader::new(File::open(full_path).unwrap());
				let image = image::io::Reader::new(&mut reader).with_guessed_format().unwrap().decode().unwrap();
				raw_pixels = image.to_rgba8().into_raw();
				dimensions = image.dimensions();	
			}
			
			// Make sure the bit depth of the texture i 24, and to do that we load the texture, then resave it
			image::save_buffer_with_format(full_path, &raw_pixels, dimensions.0, dimensions.1, image::ColorType::Rgba8, image::ImageFormat::Png);			
		}
		

		// Re-read the image, since we might've changed it's bit depth in the last scope
		let mut reader = BufReader::new(File::open(full_path).unwrap());
		let mut bytes: Vec<u8> = Vec::new();
		reader.seek(SeekFrom::Start(0));
		reader.read_to_end(&mut bytes);
		texture = Resource::Texture(LoadedTexture {
			name: String::from(""),
			width: dimensions.0 as u16,
			height: dimensions.1 as u16,
			compressed_bytes: bytes,
		});
		return texture;
	}
}
// Da code.
// Date: 2021-08-08. Warning: Do not touch this code. It will give you headaches. Trust me.
impl ResourceManager {	
	// Loads a specific resource and caches it so we can use it next time
	pub fn load_packed_resource(&mut self, name: &str, path: &str) -> Option<&Resource> {
		let name = format!("{}.{}", name, "pkg");
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
		// Get the original resource folder
		let env_path = env::current_dir().unwrap();
		let mut env_path = env_path.to_str().unwrap();
		let env_path: Vec<&str> = env_path.split("\\").collect();
		let env_path: String = format!("{}\\", &env_path[..(env_path.len() - 2)].join("\\"));
		let resources_path = format!("{}\\src\\resources\\", env_path);
		println!("Resource path '{}'", resources_path);
		
		// Recursive file finder lol
		let walk_dir = WalkDir::new(resources_path.clone());	
		// First of all, loop through all the resource directories recursively and find all the files that can be packed	
		for dir_entry in walk_dir.into_iter() {
			// Get the file
			let dir_entry = dir_entry.unwrap();
			// Skip the entry if it's not a file
			if dir_entry.path().is_dir() {
				continue;
			}
			let mut file = OpenOptions::new().read(true).open(dir_entry.path()).unwrap();
			let file_name_and_extension = dir_entry.file_name().to_str().unwrap();
			// Everything before the first dot
			let file_name = file_name_and_extension.split(".").nth(0).unwrap();
			// Everything after the first dot
			let file_extension: Vec<&str> = file_name_and_extension.split(".").collect();
			let file_extension = file_extension[1..].join(".");
			// The name where the current file is located relative to the resource's folder
			let file_path =  dir_entry.path().to_str().unwrap();
			let subdirectory_name = file_path.split(resources_path.as_str()).nth(1).unwrap().replace(file_name_and_extension, "");
			println!("Packaging file '{}{}.{}'", subdirectory_name, file_name, file_extension);

			// This is the resource that we are going to pack
			let mut resource = Resource::None;

			// Now pack each resource in it's own way
			match file_extension.as_str() {
				"vrsh.glsl" | "frsh.glsl" => {
					// This is a shader
					resource = Self::pack_shader(&file, file_extension.as_str());
				}
				"mdl3d" => {
					// This is a 3D model
					resource = Self::pack_mdl3d(&file);
				}
				"png" => {
					// This is a texture
					resource = Self::pack_texture(&mut file, &file_path);
				}
				_ => {}
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
