use byteorder::{LittleEndian, ReadBytesExt};

use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    env,
    fs::File,
    hash::{Hash, Hasher},
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    str,
};

// A resource manager that will load structs from binary files
#[derive(Default)]
pub struct ResourceManager {
    cached_resources: HashMap<u64, Resource>,
}

// A trait for structs that can be loaded from resources
pub trait LoadableResource {
    // Turn a resource into the current struct
    fn from_resource(self, resource: &Resource) -> Self;
}

// Impl block for turning all the packed data back into resources
impl ResourceManager {
    // Load back the data from the reader and turn it into a LoadedModel resource
    pub fn load_model(reader: &mut BufReader<File>) -> Option<Resource> {
        let vertices_size: u32 = reader.read_u32::<LittleEndian>().unwrap();
        let triangles_size: u32 = reader.read_u32::<LittleEndian>().unwrap();
        let mut vertices: Vec<veclib::Vector3<f32>> = Vec::new();
        let mut triangles: Vec<u32> = Vec::new();
        let mut normals: Vec<veclib::Vector3<f32>> = Vec::new();
        let mut uvs: Vec<veclib::Vector2<f32>> = Vec::new();
        let mut tangents: Vec<veclib::Vector4<f32>> = Vec::new();
        // Load the vertices
        for _ in 0..vertices_size {
            vertices.push(veclib::Vector3::<f32>::new(
                reader.read_f32::<LittleEndian>().ok()?,
                reader.read_f32::<LittleEndian>().ok()?,
                reader.read_f32::<LittleEndian>().ok()?,
            ));
        }
        // Load the normals
        for _ in 0..vertices_size {
            normals.push(veclib::Vector3::<f32>::new(
                reader.read_f32::<LittleEndian>().ok()?,
                reader.read_f32::<LittleEndian>().ok()?,
                reader.read_f32::<LittleEndian>().ok()?,
            ));
        }
        // Load the tangents
        for _ in 0..vertices_size {
            tangents.push(veclib::Vector4::<f32>::new(
                reader.read_f32::<LittleEndian>().ok()?,
                reader.read_f32::<LittleEndian>().ok()?,
                reader.read_f32::<LittleEndian>().ok()?,
                reader.read_f32::<LittleEndian>().ok()?,
            ));
        }
        // Load the uvs
        for _ in 0..vertices_size {
            uvs.push(veclib::Vector2::<f32>::new(
                reader.read_f32::<LittleEndian>().ok()?,
                reader.read_f32::<LittleEndian>().ok()?,
            ));
        }

        // Load the triangles
        for _i in 0..triangles_size {
            triangles.push(reader.read_u32::<LittleEndian>().unwrap());
        }

        Option::Some(Resource::Model(LoadedModel {
            vertices,
            normals,
            tangents,
            uvs,
            indices: triangles,
        }))
    }
    // Load back the data from the reader and turn it into a LoadedSubShader resource
    pub fn load_shader(reader: &mut BufReader<File>, local_path: String) -> Option<Resource> {
        let shader_type: u8;
        let shader_name: String;
        match reader.read_u8().ok()? {
            0 => {
                // This is a vertex subshader so the name of the shader will have a 'vertex' appended
                shader_type = 0;
                shader_name = local_path;
            }
            1 => {
                // This is a vertex subshader so the name of the shader will have a 'fragmnet' appended
                shader_type = 1;
                shader_name = local_path;
            }
            _ => {
                panic!("Shader type not supported!");
            }
        }
        // Read all the bytes until the end of the file, and then turn them into a utf8 string
        let mut bytes: Vec<u8> = Vec::new();
        reader.read_to_end(&mut bytes).unwrap();
        let shader_source = String::from_utf8(bytes).unwrap();
        Option::Some(Resource::Shader(
            LoadedSubShader {
                source: shader_source,
                subshader_type: shader_type,
            },
            shader_name,
        ))
    }
    // Load back the data from the reader and turn it into a LoadedTexture resource
    pub fn load_texture(reader: &mut BufReader<File>, local_path: String) -> Option<Resource> {
        let texture_width = reader.read_u16::<LittleEndian>().unwrap();
        let texture_height = reader.read_u16::<LittleEndian>().unwrap();
        let mut compressed_bytes: Vec<u8> = Vec::new();
        // Load all the bytes
        reader.seek(SeekFrom::Start(4)).unwrap();
        reader.read_to_end(&mut compressed_bytes).unwrap();

        // Load the bytes into the resource
        Option::Some(Resource::Texture(
            LoadedTexture {
                width: texture_width,
                height: texture_height,
                compressed_bytes,
            },
            local_path,
        ))
    }
}

// Da code.
// Date: 2021-08-08. Warning: Do not touch this code. It will give you headaches. Trust me.
// Date: 2021-08-13. Basically rewrote the whole thing. It's good now
impl ResourceManager {
    // Loads a specific resource and caches it so we can use it next time
    pub fn load_packed_resource(&mut self, local_path: &str) -> Result<&Resource, hypo_errors::ResourceError> {
        println!("{}", local_path);
        // Get the global path of the packed-resources folder
        let exe_path = env::current_exe().unwrap();
        let exe_path = exe_path.to_str().ok_or(hypo_errors::ResourceError::new_str("Exe path not valid!"))?;
        let client_folder: Vec<&str> = exe_path.split('\\').collect();
        let client_folder = format!("{}\\", &client_folder[..(client_folder.len() - 1)].join("\\"));
        let packed_resources_path = format!("{}packed-resources\\", client_folder);

        // Now split the local path into the extension and name
        let name: Vec<&str> = local_path.split('\\').collect();
        let name_and_extension = name[name.len() - 1];
        let _name = name_and_extension
            .split('.')
            .next()
            .ok_or(hypo_errors::ResourceError::new(format!(
                "Name or extension are not valid for resource file '{}'",
                local_path
            )))?
            .to_string();
        let extension: Vec<&str> = name_and_extension.split('.').collect();
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
            let resource = self.cached_resources.get(&hashed_name).unwrap();
            return Ok(resource);
        }

        // The global file path for the hashed packed resource
        let file_path = format!("{}{}.pkg", packed_resources_path, hashed_name);
        let mut resource: Resource = Resource::None;

        // Since the resource was not in the cache, load it and then put it in the cache
        // Open the file
        let packed_file = File::open(file_path)
            .ok()
            .ok_or(hypo_errors::ResourceError::new(format!("Resource file '{}' could not be opened!", local_path)))?;
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

        Ok(resource)
    }
    // Unloads a resource to save on memory
    pub fn unload_resouce(&mut self) {}
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
    pub vertices: Vec<veclib::Vector3<f32>>,
    pub normals: Vec<veclib::Vector3<f32>>,
    pub tangents: Vec<veclib::Vector4<f32>>,
    pub uvs: Vec<veclib::Vector2<f32>>,
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
    pub subshader_type: u8,
}
// A sound effect that can be played at any time
pub struct LoadedSoundEffect {}
