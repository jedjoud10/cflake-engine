use byteorder::{LittleEndian, ReadBytesExt};

use errors::ResourceError;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    env,
    fs::File,
    hash::{Hash, Hasher},
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    str,
};

// Should we logs resource events?
pub const DEBUG_LOGS: bool = true;

// A resource manager that will load structs from binary files
#[derive(Default)]
pub struct ResourceManager {
    cached_resources: HashMap<u64, Resource>,
}

// A trait for structs that can be loaded from resources
pub trait LoadableResource {
    // Turn a resource into the current struct
    fn from_resource(self, resource: &Resource) -> Option<Self>
    where
        Self: Sized;
    // Load this resource directly from a path, this is implemented by default
    fn from_path(self, local_path: &str, resource_manager: &mut ResourceManager) -> Option<Self>
    where
        Self: Sized,
    {
        let resource = resource_manager.load_packed_resource(local_path).ok()?;
        return Self::from_resource(self, resource);
    }
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
        let shader_type: u8 = reader.read_u8().ok()?;
        let shader_name = local_path;
        match shader_type {
            0 | 1 | 2 => {}
            _ => {
                panic!("Shader type not supported!")
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
    // Load back a font
    pub fn load_font(reader: &mut BufReader<File>, name: String) -> Option<Resource> {
        // Read the custom font
        let mut output_font = LoadedFont {
            dimensions: veclib::Vector2::ZERO,
            texture_pixels: Vec::new(),
            chars: Vec::new(),
        };

        // Get the width and height of the bitmap
        let width = reader.read_u32::<LittleEndian>().unwrap();
        let height = reader.read_u32::<LittleEndian>().unwrap();

        // Read the pixels, one by one
        for i in 0..(width * height) {
            let pixel = reader.read_u8().unwrap();
            output_font.texture_pixels.push(pixel);
        }

        // Get the number of ASCII characters we have
        let font_char_num = reader.read_u8().unwrap();

        // Read the chars
        for i in 0..font_char_num {
            // Get the data back
            let loaded_char = LoadedChar {
                id: reader.read_u8().unwrap(),
                min: veclib::Vector2::new(reader.read_u32::<LittleEndian>().unwrap(), reader.read_u32::<LittleEndian>().unwrap()),
                max: veclib::Vector2::new(reader.read_u32::<LittleEndian>().unwrap(), reader.read_u32::<LittleEndian>().unwrap()),
            };
            output_font.chars.push(loaded_char);
        }
        Some(Resource::Font(output_font, name))
    }
}

// Da code.
// Date: 2021-08-08. Warning: Do not touch this code. It will give you headaches. Trust me.
// Date: 2021-08-13. Basically rewrote the whole thing. It's good now
impl ResourceManager {
    // Get the hashed name from the local path
    fn calculate_hashed_name(local_path: &str) -> u64 {
        {
            let mut hasher = DefaultHasher::new();
            local_path.hash(&mut hasher);
            hasher.finish()
        }
    }
    // Turn a local path into a literal, hashed path
    pub fn local_to_global_path(local_path: &str) -> Result<(String, String, u64), ResourceError> {
        // Get the global path of the packed-resources folder
        let exe_path = env::current_exe().unwrap();
        let exe_path = exe_path.to_str().ok_or(ResourceError::new_str("Exe path not valid!"))?;
        let client_folder: Vec<&str> = exe_path.split('\\').collect();
        let client_folder = format!("{}\\", &client_folder[..(client_folder.len() - 1)].join("\\"));
        let packed_resources_path = format!("{}packed-resources\\", client_folder);

        // Now split the local path into the extension and name
        let name: Vec<&str> = local_path.split('\\').collect();
        let name_and_extension = name[name.len() - 1];
        let _name = name_and_extension
            .split('.')
            .next()
            .ok_or(ResourceError::new(format!("Name or extension are not valid for resource file '{}'", local_path)))?
            .to_string();
        let extension: Vec<&str> = name_and_extension.split('.').collect();
        let extension = extension[1..].join(".");
        // Hash the local path and then use it to load the file
        let hashed_name: u64 = Self::calculate_hashed_name(local_path);

        // The global file path for the hashed packed resource
        let file_path = format!("{}{}.pkg", packed_resources_path, hashed_name);
        return Ok((file_path, extension, hashed_name));
    }
    // Loads a specific resource and caches it so we can use it next time
    pub fn load_packed_resource(&mut self, local_path: &str) -> Result<&Resource, ResourceError> {
        if DEBUG_LOGS {
            println!("Loading resource: '{}'...", local_path);
        }
        let (file_path, extension, hashed_name) = Self::local_to_global_path(local_path)?;
        // Check if we have the file cached, if we do, then just take the resource from the cache
        if self.cached_resources.contains_key(&hashed_name) {
            // We have the needed resource in the resource cache!
            let resource = self.cached_resources.get(&hashed_name).unwrap();
            if DEBUG_LOGS {
                println!("Loaded resource: '{}' from cache succsessfully!", local_path);
            }
            return Ok(resource);
        }
        let mut resource: Resource = Resource::None;

        // Since the resource was not in the cache, load it and then put it in the cache
        // Open the file
        let packed_file = File::open(file_path)
            .ok()
            .ok_or(ResourceError::new(format!("Resource file '{}' could not be opened!", local_path)))?;
        let mut reader = BufReader::new(packed_file);

        // Update the resource type
        match extension.as_str() {
            "vrsh.glsl" | "frsh.glsl" | "cmpt.glsl" => {
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
            "font" => {
                // This is a font
                resource = Self::load_font(&mut reader, local_path.to_string()).unwrap();
            }
            _ => {}
        }

        // Insert the resource in the cache
        self.cached_resources.insert(hashed_name, resource);
        let resource = self.cached_resources.get(&hashed_name).unwrap();
        if DEBUG_LOGS {
            println!("Loaded resource: '{}' succsessfully!", local_path);
        }
        Ok(resource)
    }
    // Unloads a resource to save on memory
    pub fn unload_resouce(&mut self, local_path: &str) {
        // Get the hashed name
        let hashed_name: u64 = Self::calculate_hashed_name(local_path);
        // Unload the resource from cache
        self.cached_resources.remove(&hashed_name);
    }
    // Load the literal lines from a packed resource, with a byte padding at the start (Useful for function shaders)
    pub fn load_lines_packed_resource(&mut self, local_path: &str, byte_padding: u64) -> Result<String, ResourceError> {
        // Get the global hashed path file
        let (file_path, extension, hashed_name) = Self::local_to_global_path(local_path).unwrap();
        // Open the file first
        let packed_file = File::open(file_path)
            .ok()
            .ok_or(ResourceError::new(format!("Resource file '{}' could not be opened!", local_path)))?;
        let mut reader = BufReader::new(packed_file);

        // Offset the reader
        reader.seek(SeekFrom::Start(byte_padding)).unwrap();
        let lines: Vec<String> = reader.lines().map(|x| x.unwrap()).collect();
        // Now combine all the lines into the text file
        let text = lines.join("\n");
        return Ok(text);
    }
}

// A simple loaded resource
// TODO: Handle the caching internally so we can get rid of the Texture and Shader cachers
pub enum Resource {
    None,
    Model(LoadedModel),
    Texture(LoadedTexture, String),
    Shader(LoadedSubShader, String),
    Sound(LoadedSoundEffect),
    Font(LoadedFont, String),
    // Only used if we are not actually doing packing, just passing the bytes from the normal resource to the packed one
    Bytes(Vec<u8>),
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

// A loaded char font, just contains the padding for each character and such
pub struct LoadedChar {
    pub id: u8,
    pub min: veclib::Vector2<u32>,
    pub max: veclib::Vector2<u32>,
}

// A loaded font
pub struct LoadedFont {
    pub dimensions: veclib::Vector2<u32>,
    pub texture_pixels: Vec<u8>,
    pub chars: Vec<LoadedChar>,
}
