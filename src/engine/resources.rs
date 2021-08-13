use crate::engine::rendering::shader::SubShaderType;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use image::GenericImageView;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    env,
    ffi::OsStr,
    fmt::format,
    fs::{create_dir, create_dir_all, read_dir, remove_file, File, OpenOptions},
    hash::{Hash, Hasher},
    io::{BufRead, BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write},
    os::windows::prelude::MetadataExt,
    path::{Path, PathBuf},
    str,
    thread::current,
    time::{Duration, SystemTime},
};
use walkdir::WalkDir;

// A resource manager that will load structs from binary files
#[derive(Default)]
pub struct ResourceManager {
    cached_resources: HashMap<u64, Resource>,
}

// All the conversion stuff from File -> Resource
impl ResourceManager {
    // Turn a mdl3d file into a LoadedModel resource
    pub fn convert_mdl3d(file: &File) -> Resource {
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
                    tangents.push(glam::vec4(coords[0], coords[1], coords[2], coords[3]));
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
    pub fn convert_shader(file: &File, extension: &str) -> Resource {
        // The shader resource
        let mut shader: Resource = Resource::None;
        // String holding the extension of the file
        let mut reader = BufReader::new(file);
        match extension {
            "vrsh.glsl" => {
                // This is a vertex shader
                let mut string_source: String = String::new();
                reader.read_to_string(&mut string_source);
                shader = Resource::Shader(
                    LoadedSubShader {
                        source: string_source,
                        subshader_type: SubShaderType::Vertex,
                    },
                    String::new(),
                );
            }
            "frsh.glsl" => {
                // This is a fragment shader
                let mut string_source: String = String::new();
                reader.read_to_string(&mut string_source);
                shader = Resource::Shader(
                    LoadedSubShader {
                        source: string_source,
                        subshader_type: SubShaderType::Fragment,
                    },
                    String::new(),
                );
            }
            _ => {}
        }
        return shader;
    }
    // Turn a texture file to a LoadedTexture resource
    // While we're at it, make sure the texture has an alpha channel and EXACTLY a 32 bit depth
    pub fn convert_texture(file: &mut File, full_path: &str) -> Resource {
        // The texture resource
        let mut texture: Resource = Resource::None;
        let mut dimensions: (u32, u32) = (0, 0);
        // Check if we even need to update the image
        let should_update: bool = {
            let mut reader = BufReader::new(file);
            let image = image::io::Reader::new(&mut reader)
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap();
            match image {
                image::DynamicImage::ImageRgba8(_) => {
                    // No need to do anything since we already have this texture at 32 bits per pixel
                    false
                }
                _ => {
                    // Anything other than 32 bits
                    true
                }
            }
        };
        if should_update {
            // We need to make this it's own scope because we cannot have a reader and a writer at the same time
            let mut raw_pixels: Vec<u8> = Vec::new();
            {
                let mut reader = BufReader::new(File::open(full_path).unwrap());
                let image = image::io::Reader::new(&mut reader)
                    .with_guessed_format()
                    .unwrap()
                    .decode()
                    .unwrap();
                raw_pixels = image.to_rgba8().into_raw();
                dimensions = image.dimensions();
            }

            // Make sure the bit depth of the texture i 32, and to do that we load the texture, then resave it
            image::save_buffer_with_format(
                full_path,
                &raw_pixels,
                dimensions.0,
                dimensions.1,
                image::ColorType::Rgba8,
                image::ImageFormat::Png,
            );
        } else {
            // I forgot to tell it to get the dimensions of the texture even if we shouldn't resave it -.-
            let mut reader = BufReader::new(File::open(full_path).unwrap());
            let image = image::io::Reader::new(&mut reader)
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap();
            dimensions = image.dimensions();
        }

        // Re-read the image, since we might've changed it's bit depth in the last scope
        let mut reader = BufReader::new(File::open(full_path).unwrap());
        let mut bytes: Vec<u8> = Vec::new();
        reader.seek(SeekFrom::Start(0));
        reader.read_to_end(&mut bytes);
        texture = Resource::Texture(
            LoadedTexture {
                width: dimensions.0 as u16,
                height: dimensions.1 as u16,
                compressed_bytes: bytes,
            },
            String::new(),
        );
        return texture;
    }
}

// Impl block for packing all the Resources into a file
impl ResourceManager {
    // Pack a LoadedModel resource into a file
    pub fn pack_model(writer: &mut BufWriter<File>, resource: Resource) -> std::io::Result<()> {
        let mut model: LoadedModel;
        match resource {
            Resource::Model(__model) => {
                model = __model;
            }
            _ => {
                panic!("Resource was not a model!");
            }
        }
        // Write to the strem
        writer.write_u32::<LittleEndian>(model.vertices.len() as u32)?;
        writer.write_u32::<LittleEndian>(model.indices.len() as u32)?;
        // Write the vertices
        for vertex in model.vertices {
            writer.write_f32::<LittleEndian>(vertex.x)?;
            writer.write_f32::<LittleEndian>(vertex.y)?;
            writer.write_f32::<LittleEndian>(vertex.z)?;
        }
        // Write the normals
        for normal in model.normals {
            writer.write_f32::<LittleEndian>(normal.x)?;
            writer.write_f32::<LittleEndian>(normal.y)?;
            writer.write_f32::<LittleEndian>(normal.z)?;
        }
        // Write the tangents
        for tangent in model.tangents {
            writer.write_f32::<LittleEndian>(tangent.x)?;
            writer.write_f32::<LittleEndian>(tangent.y)?;
            writer.write_f32::<LittleEndian>(tangent.z)?;
            writer.write_f32::<LittleEndian>(tangent.w)?;
        }
        // Write the uvs
        for uv in model.uvs {
            writer.write_f32::<LittleEndian>(uv.x)?;
            writer.write_f32::<LittleEndian>(uv.y)?;
        }
        // Write the indices
        for index in model.indices {
            writer.write_u32::<LittleEndian>(index)?;
        }

        return std::io::Result::Ok(());
    }
    // Pack a LoadedSubShader resource into a file
    pub fn pack_shader(writer: &mut BufWriter<File>, resource: Resource) -> std::io::Result<()> {
        let mut shader: LoadedSubShader;
        match resource {
            Resource::Shader(__shader, _) => {
                shader = __shader;
            }
            _ => {
                panic!("Resource was not a shader!");
            }
        }

        // Turn the source string into bytes, and write them into the resource file
        let mut string_bytes = shader.source.into_bytes().to_vec();
        let mut shader_type_byte: u8 = 0;
        // Save the type of this subshader, can either be a Vertex or a Fragment subshader
        match shader.subshader_type {
            SubShaderType::Vertex => shader_type_byte = 0,
            SubShaderType::Fragment => shader_type_byte = 1,
        }
        // Write the type of subshader
        writer.write_u8(shader_type_byte)?;

        // Write all the bytes
        for byte in string_bytes {
            writer.write_u8(byte)?;
        }
        return std::io::Result::Ok(());
    }
    // Pack a LoadedTexture resource into a file
    pub fn pack_texture(writer: &mut BufWriter<File>, resource: Resource) -> std::io::Result<()> {
        let mut texture: LoadedTexture;
        match resource {
            Resource::Texture(__texture, _) => {
                texture = __texture;
            }
            _ => {
                panic!("Resource was not a texture!");
            }
        }

        // Write the dimensions of the texture
        writer.write_u16::<LittleEndian>(texture.width)?;
        writer.write_u16::<LittleEndian>(texture.height)?;

        // Now write all the bytes
        for byte in texture.compressed_bytes {
            writer.write_u8(byte)?;
        }
        return std::io::Result::Ok(());
    }
}

// Impl block for turning all the packed data back into resources
impl ResourceManager {
    // Load back the data from the reader and turn it into a LoadedModel resource
    pub fn load_model(reader: &mut BufReader<File>) -> Option<Resource> {
        let vertices_size: u32 = reader.read_u32::<LittleEndian>().unwrap();
        let triangles_size: u32 = reader.read_u32::<LittleEndian>().unwrap();
        let mut vertices: Vec<glam::Vec3> = Vec::new();
        let mut triangles: Vec<u32> = Vec::new();
        let mut normals: Vec<glam::Vec3> = Vec::new();
        let mut uvs: Vec<glam::Vec2> = Vec::new();
        let mut tangents: Vec<glam::Vec4> = Vec::new();
        // Load the vertices
        for _ in 0..vertices_size {
            vertices.push(glam::vec3(
                reader.read_f32::<LittleEndian>().ok()?,
                reader.read_f32::<LittleEndian>().ok()?,
                reader.read_f32::<LittleEndian>().ok()?,
            ));
        }
        // Load the normals
        for _ in 0..vertices_size {
            normals.push(glam::vec3(
                reader.read_f32::<LittleEndian>().ok()?,
                reader.read_f32::<LittleEndian>().ok()?,
                reader.read_f32::<LittleEndian>().ok()?,
            ));
        }
        // Load the tangents
        for _ in 0..vertices_size {
            tangents.push(glam::vec4(
                reader.read_f32::<LittleEndian>().ok()?,
                reader.read_f32::<LittleEndian>().ok()?,
                reader.read_f32::<LittleEndian>().ok()?,
                reader.read_f32::<LittleEndian>().ok()?,
            ));
        }
        // Load the uvs
        for _ in 0..vertices_size {
            uvs.push(glam::vec2(
                reader.read_f32::<LittleEndian>().ok()?,
                reader.read_f32::<LittleEndian>().ok()?,
            ));
        }

        // Load the triangles
        for _i in 0..triangles_size {
            triangles.push(reader.read_u32::<LittleEndian>().unwrap());
        }

        return Option::Some(Resource::Model(LoadedModel {
            vertices,
            normals,
            tangents,
            uvs,
            indices: triangles,
        }));
    }
    // Load back the data from the reader and turn it into a LoadedSubShader resource
    pub fn load_shader(reader: &mut BufReader<File>, local_path: String) -> Option<Resource> {
        let shader_type: SubShaderType;
        let mut shader_name: String = String::new();
        match reader.read_u8().ok()? {
            0 => {
                // This is a vertex subshader so the name of the shader will have a 'vertex' appended
                shader_type = SubShaderType::Vertex;
                shader_name = format!("{}.{}", &local_path, "vrsh.glsl");
            }
            1 => {
                // This is a vertex subshader so the name of the shader will have a 'fragmnet' appended
                shader_type = SubShaderType::Fragment;
                shader_name = format!("{}.{}", &local_path, "frsh.glsl");
            }
            _ => {
                panic!("Shader type not supported!");
            }
        }
        // Read all the bytes until the end of the file, and then turn them into a utf8 string
        let mut bytes: Vec<u8> = Vec::new();
        reader.read_to_end(&mut bytes);
        let shader_source = String::from_utf8(bytes).unwrap();
        return Option::Some(Resource::Shader(
            LoadedSubShader {
                source: shader_source.clone(),
                subshader_type: shader_type.clone(),
            },
            shader_name,
        ));
    }
    // Load back the data from the reader and turn it into a LoadedTexture resource
    pub fn load_texture(reader: &mut BufReader<File>, local_path: String) -> Option<Resource> {
        let texture_width = reader.read_u16::<LittleEndian>().unwrap();
        let texture_height = reader.read_u16::<LittleEndian>().unwrap();
        let mut compressed_bytes: Vec<u8> = Vec::new();
        // Load all the bytes
        reader.seek(SeekFrom::Start(4)).unwrap();
        reader.read_to_end(&mut compressed_bytes);

        // Load the bytes into the resource
        return Option::Some(Resource::Texture(
            LoadedTexture {
                width: texture_width,
                height: texture_height,
                compressed_bytes,
            },
            local_path,
        ));
    }
}

// Da code.
// Date: 2021-08-08. Warning: Do not touch this code. It will give you headaches. Trust me.
// Date: 2021-08-13. Basically rewrote the whole thing. It's good now
impl ResourceManager {
    // Loads a specific resource and caches it so we can use it next time
    pub fn load_packed_resource(&mut self, local_path: &str) -> Option<&Resource> {
        // Get the global path of the packed-resources folder
        let exe_path = env::current_exe().unwrap();
        let exe_path = exe_path.to_str().unwrap();
        let client_folder: Vec<&str> = exe_path.split("\\").collect();
        let client_folder = format!(
            "{}\\",
            &client_folder[..(client_folder.len() - 1)].join("\\")
        );
        let packed_resources_path = format!("{}packed-resources\\", client_folder);

        // Now split the local path into the extension and name
        let name: Vec<&str> = local_path.split("\\").collect();
        let name_and_extension = name[name.len() - 1];
        let name = name_and_extension.split(".").nth(0).unwrap().to_string();
        let extension: Vec<&str> = name_and_extension.split(".").collect();
        let extension = extension[1..].join(".");
        // Hash the local path and then use it to load the file
        let hashed_name: u64 = {
            let mut hasher = DefaultHasher::new();
            local_path.hash(&mut hasher);
            hasher.finish()
        };
        // Check if we have the file cached, if we do, then just take the resource from the cache
        if self.cached_resources.contains_key(&hashed_name) {
            // We have the needed resource in the resource cache!
            let resource = self.cached_resources.get(&hashed_name)?;
            println!("Load cached resource: '{}'", local_path);
            Some(resource);
        }

        // The global file path for the hashed packed resource
        let file_path = format!("{}{}.pkg", packed_resources_path, hashed_name);
        let mut resource: Resource = Resource::None;

        // Since the resource was not in the cache, load it and then put it in the cache
        // Open the file
        println!("{}", local_path);
        let packed_file = File::open(file_path).unwrap();
        let mut reader = BufReader::new(packed_file);

        // Update the resource type
        match extension.as_str() {
            "vrsh.glsl" | "frsh.glsl" => {
                // Load the packed resource as a subshader
                resource = Self::load_shader(&mut reader, local_path.to_string()).unwrap();
            }
            "mdl3d" => {
                // Load the packed resource as a model
                resource = Self::load_model(&mut reader).unwrap();
            }
            "png" => {
                // This is a texture
                resource = Self::load_texture(&mut reader, local_path.to_string()).unwrap();
            }
            _ => {}
        }

        // Insert the resource in the cache
        self.cached_resources.insert(hashed_name, resource);
        let resource = self.cached_resources.get(&hashed_name).unwrap();
        println!("Cached resource: '{}'", local_path);

        return Some(resource);
    }
    // Unloads a resource to save on memory
    pub fn unload_resouce(&mut self) {}
    // Saves all the resources from the "resources" folder into the "packed-resources" folder
    pub fn pack_resources() -> Option<()> {
        // Get the original resource folder
        let env_path = env::current_dir().unwrap();
        let mut env_path = env_path.to_str().unwrap();
        let env_path: Vec<&str> = env_path.split("\\").collect();
        let env_path: String = format!("{}\\", &env_path[..(env_path.len() - 2)].join("\\"));
        let resources_path = format!("{}src\\resources\\", env_path);
        let packed_resources_path = format!("{}src\\packed-resources\\", env_path);
        println!("Resource path '{}'", resources_path);

        // Make the log file that will be used later to save time when packing resources
        let log_file_path = format!("{}log.log", packed_resources_path);
        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(log_file_path.clone())
            .unwrap();
        // A hashmap containing all the packed resources in the log file, with the timestamps of when their last edit happened
        let mut log_file_packed_timestamps: HashMap<u64, u64> = HashMap::new();
        {
            let mut log_file_reader = BufReader::new(log_file);
            // An index to keep track what 8 bytes we are reading
            //
            // Read the log file
            loop {
                // There is a possibility for the first 8 bytes that we read in this iteration to cause an error (EoF), so we gotta handle it properly
                let hashed_name = log_file_reader.read_u64::<LittleEndian>();
                let hashed_name = match hashed_name {
                    Ok(val) => {
                        // We read the value properly
                        val
                    }
                    Err(_) => {
                        // Break out of the loop since we can't read anymore
                        break;
                    }
                };
                let timestamp = log_file_reader.read_u64::<LittleEndian>().ok()?;
                // Add the data to the hashmap
                log_file_packed_timestamps.insert(hashed_name, timestamp);
            }
        }

        println!("{:?}", log_file_packed_timestamps);

        // Reopen the file since it's a moved value
        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(log_file_path.clone())
            .unwrap();
        let mut log_file_writer = BufWriter::new(log_file);

        // Keep track of all the resources in the original resources folder
        let mut hashed_names: Vec<u64> = Vec::new();
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
            let mut file = OpenOptions::new()
                .read(true)
                .open(dir_entry.path())
                .unwrap();
            let file_metadata = file.metadata().ok()?;
            let file_name_and_extension = dir_entry.file_name().to_str().unwrap();
            // Everything before the first dot
            let file_name = file_name_and_extension.split(".").nth(0).unwrap();
            // Everything after the first dot
            let file_extension: Vec<&str> = file_name_and_extension.split(".").collect();
            let file_extension = file_extension[1..].join(".");
            // The name where the current file is located relative to the resource's folder
            let file_path = dir_entry.path().to_str().unwrap();
            let subdirectory_name = file_path
                .split(resources_path.as_str())
                .nth(1)
                .unwrap()
                .replace(file_name_and_extension, "");
            println!(
                "Packing file '{}{}.{}'",
                subdirectory_name, file_name, file_extension
            );

            // This is the resource that we are going to pack
            let mut resource = Resource::None;

            // Create a hashed name to make it able for all the resources to be in one folder
            let packed_file_hashed_name: u64 = {
                let mut hasher = DefaultHasher::new();
                // Use the resource as the hash "key"
                format!("{}{}.{}", subdirectory_name, file_name, file_extension).hash(&mut hasher);
                hasher.finish()
            };

            // Keep it in sync
            hashed_names.push(packed_file_hashed_name);

            // Check if we even need to pack this resource
            if log_file_packed_timestamps.contains_key(&packed_file_hashed_name) {
                // We already packed this file, but we need to check if the original resource file was changed
                let packed_timestamp = log_file_packed_timestamps.get(&packed_file_hashed_name)?;
                let resource_timestamp = file_metadata
                    .modified()
                    .unwrap()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                // Did we edit the file?
                if resource_timestamp > packed_timestamp.clone() {
                    // We did edit the file, so we need to pack it
                } else {
                    // We didn't edit the file, no need to pack
                    println!("File was not changed, skipped packing!");
                    continue;
                }
            } else {
                // The file was just added to the resources folder, so we gotta pack it
            }

            // Now convert each resource in it's own way
            match file_extension.as_str() {
                "vrsh.glsl" | "frsh.glsl" => {
                    // This is a shader
                    resource = Self::convert_shader(&file, file_extension.as_str());
                }
                "mdl3d" => {
                    // This is a 3D model
                    resource = Self::convert_mdl3d(&file);
                }
                "png" => {
                    // This is a texture
                    resource = Self::convert_texture(&mut file, &file_path);
                }
                _ => {}
            }

            // Now time to actually pack the resource
            let packed_file_path =
                format!("{}{}.pkg", packed_resources_path, packed_file_hashed_name);
            // Create the file
            let packed_file = File::create(packed_file_path).unwrap();
            let packed_file_metadata = packed_file.metadata().ok()?;
            let last_time_packed = packed_file_metadata
                .modified()
                .unwrap()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let mut writer = BufWriter::new(packed_file);
            match resource {
                Resource::Shader(_, _) => {
                    // This is a shader
                    Self::pack_shader(&mut writer, resource);
                }
                Resource::Model(_) => {
                    // This a 3D model
                    Self::pack_model(&mut writer, resource);
                }
                Resource::Texture(_, _) => {
                    // This a texture
                    Self::pack_texture(&mut writer, resource);
                }
                _ => {}
            }

            // After packing the file, we need to append it to log file
            log_file_packed_timestamps.insert(packed_file_hashed_name, last_time_packed);
        }

        // Check if there are files in the packed hashed names timestamp hashmap that don't exist in the resource folder
        log_file_packed_timestamps.retain(|hashed, _| {
            // Only keep the hashed names that are actually exist in the original resource folder
            let keep = hashed_names.contains(&hashed);
            if !keep {
                let packed_file_path = format!("{}{}.pkg", packed_resources_path, hashed);
                println!("{}", packed_file_path);
                remove_file(packed_file_path);
            }
            keep
        });

        // Rewrite all the hashed names and timestamps that we saved in the hashmap
        for (name, timestamp) in log_file_packed_timestamps {
            log_file_writer.write_u64::<LittleEndian>(name).ok()?;
            log_file_writer.write_u64::<LittleEndian>(timestamp).ok()?;
        }

        // Packed the resources sucsessfully
        return Some(());
    }
}

// A simple loaded resource
pub enum Resource {
    None,
    Model(LoadedModel),
    Texture(LoadedTexture, String),
    Shader(LoadedSubShader, String),
    Sound(LoadedSoundEffect, String),
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
    pub width: u16,
    pub height: u16,
    pub compressed_bytes: Vec<u8>,
}
// A loaded sub shader
#[derive(Clone)]
pub struct LoadedSubShader {
    pub source: String,
    pub subshader_type: SubShaderType,
}
// A sound effect that can be played at any time
pub struct LoadedSoundEffect {}
