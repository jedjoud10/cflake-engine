use std::{collections::{hash_map::DefaultHasher, HashMap}, env, fs::{remove_file, File, OpenOptions}, hash::{Hash, Hasher}, io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write}, path::Path, process::Command, time::SystemTime};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use image::{EncodableLayout, GenericImageView};
use resources::*;
use walkdir::WalkDir;

// Time for the resource packer
// Turn a mdl3d file into a LoadedModel resource
pub fn convert_mdl3d(file: &File) -> Resource {
    // Create all the buffers
    let reader = BufReader::new(file);
    let mut vertices: Vec<veclib::Vector3<f32>> = Vec::new();
    let mut normals: Vec<veclib::Vector3<f32>> = Vec::new();
    let mut tangents: Vec<veclib::Vector4<f32>> = Vec::new();
    let mut uvs: Vec<veclib::Vector2<f32>> = Vec::new();
    let mut triangles: Vec<u32> = Vec::new();
    // Go over each line and scan it
    for line in reader.lines() {
        let line = line.unwrap();
        let start = line.split_once(" ").unwrap().0;
        let other = line.split_once(" ").unwrap().1;
        match start {
            // Vertices
            "v" => {
                let coords: Vec<f32> = other.split('/').map(|coord| coord.parse::<f32>().unwrap()).collect();
                vertices.push(veclib::Vector3::new(coords[0], coords[1], coords[2]));
            }
            // Normals
            "n" => {
                let coords: Vec<f32> = other.split('/').map(|coord| coord.parse::<f32>().unwrap()).collect();
                normals.push(veclib::Vector3::new(coords[0], coords[1], coords[2]));
            }
            // UVs
            "u" => {
                let coords: Vec<f32> = other.split('/').map(|coord| coord.parse::<f32>().unwrap()).collect();
                uvs.push(veclib::Vector2::new(coords[0], coords[1]));
            }
            // Tangents
            "t" => {
                let coords: Vec<f32> = other.split('/').map(|coord| coord.parse::<f32>().unwrap()).collect();
                tangents.push(veclib::Vector4::new(coords[0], coords[1], coords[2], coords[3]));
            }
            // Triangle indices
            "i" => {
                // Split the triangle into 3 indices
                let mut indices = other.split('/').map(|x| x.to_string().parse::<u32>().unwrap()).collect();
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
    Resource::Model(model)
}
// Turn a shader file of any type (vertex, fragment, etc) to a LoadedShader resource
pub fn convert_shader(file: &File, extension: &str) -> Resource {
    // The shader resource
    let mut shader: Resource = Resource::None;
    // String holding the extension of the file
    let mut reader = BufReader::new(file);
    let subshader_type: i8;
    match extension {
        "vrsh.glsl" => subshader_type = 0,
        "frsh.glsl" => subshader_type = 1,
        "cmpt.glsl" => subshader_type = 2,
        "func.glsl" => subshader_type = 3,
        _ => subshader_type = -1,
    }
    // Check if the subshader is even valid in the first place
    if subshader_type == -1 {
        panic!("Invalid subshader type!")
    }
    // Convert the shader
    let mut string_source: String = String::new();
    reader.read_to_string(&mut string_source).unwrap();
    shader = Resource::Shader(
        LoadedSubShader {
            source: string_source,
            subshader_type: subshader_type as u8,
        },
        String::new(),
    );
    shader
}
// Turn a texture file to a LoadedTexture resource
// While we're at it, make sure the texture has an alpha channel and EXACTLY a 32 bit depth
pub fn convert_texture(file: &mut File, full_path: &str) -> Resource {
    // The texture resource
    let texture: Resource;
    let dimensions: (u32, u32);
    // Check if we even need to update the image
    let should_update: bool = {
        let mut reader = BufReader::new(file);
        let image = image::io::Reader::new(&mut reader).with_guessed_format().unwrap().decode().unwrap();
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
        let raw_pixels: Vec<u8>;
        {
            let mut reader = BufReader::new(File::open(full_path).unwrap());
            let image = image::io::Reader::new(&mut reader).with_guessed_format().unwrap().decode().unwrap();
            raw_pixels = image.to_rgba8().into_raw();
            dimensions = image.dimensions();
        }

        // Make sure the bit depth of the texture i 32, and to do that we load the texture, then resave it
        image::save_buffer_with_format(full_path, &raw_pixels, dimensions.0, dimensions.1, image::ColorType::Rgba8, image::ImageFormat::Png).unwrap();
    } else {
        // I forgot to tell it to get the dimensions of the texture even if we shouldn't resave it -.-
        let mut reader = BufReader::new(File::open(full_path).unwrap());
        let image = image::io::Reader::new(&mut reader).with_guessed_format().unwrap().decode().unwrap();
        dimensions = image.dimensions();
    }

    // Re-read the image, since we might've changed it's bit depth in the last scope
    let mut reader = BufReader::new(File::open(full_path).unwrap());
    let mut bytes: Vec<u8> = Vec::new();
    reader.seek(SeekFrom::Start(0)).unwrap();
    reader.read_to_end(&mut bytes).unwrap();
    texture = Resource::Texture(
        LoadedTexture {
            width: dimensions.0 as u16,
            height: dimensions.1 as u16,
            compressed_bytes: bytes,
        },
        String::new(),
    );
    texture
}

// Pack a LoadedModel resource into a file
pub fn pack_model(writer: &mut BufWriter<File>, model: LoadedModel) -> std::io::Result<()> {
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

    std::io::Result::Ok(())
}
// Pack a LoadedSubShader resource into a file
pub fn pack_shader(writer: &mut BufWriter<File>, shader: LoadedSubShader) -> std::io::Result<()> {
    // Turn the source string into bytes, and write them into the resource file
    let string_bytes = shader.source.into_bytes().to_vec();
    // Save the type of this subshader, can either be a Vertex or a Fragment subshader
    let shader_type_byte = shader.subshader_type;

    // Write the type of subshader
    writer.write_u8(shader_type_byte)?;

    // Write all the bytes
    for byte in string_bytes {
        writer.write_u8(byte)?;
    }
    std::io::Result::Ok(())
}
// Pack a LoadedTexture resource into a file
pub fn pack_texture(writer: &mut BufWriter<File>, texture: LoadedTexture) -> std::io::Result<()> {
    // Write the dimensions of the texture
    writer.write_u16::<LittleEndian>(texture.width)?;
    writer.write_u16::<LittleEndian>(texture.height)?;

    // Now write all the bytes
    for byte in texture.compressed_bytes {
        writer.write_u8(byte)?;
    }
    std::io::Result::Ok(())
}
// Saves all the resources from the "resources" folder into the "packed-resources" folder
pub fn pack_resources(src_path: String) -> Option<()> {
    // Get the original resource folder
    let env_path = src_path.clone();
    let env_path: Vec<&str> = env_path.split('\\').collect();
    let _env_path: String = format!("{}\\", &env_path[..(env_path.len() - 2)].join("\\"));
    let resources_path = format!("{}\\resources\\resources\\", src_path);
    let packed_resources_path = format!("{}\\resources\\packed-resources\\", src_path);
    println!("Resource path '{}'", resources_path);
    // Make the log file that will be used later to save time when packing resources
    let log_file_path = format!("{}log.log", packed_resources_path);
    println!("Log file path '{}'", log_file_path);
    let log_file = OpenOptions::new().create(true).write(true).read(true).open(log_file_path.clone()).unwrap();
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
    let log_file = OpenOptions::new().create(true).write(true).read(true).open(log_file_path).unwrap();
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
        let mut file = OpenOptions::new().read(true).open(dir_entry.path()).unwrap();
        let file_metadata = file.metadata().ok()?;
        let file_name_and_extension = dir_entry.file_name().to_str().unwrap();
        // Everything before the first dot
        let file_name = file_name_and_extension.split('.').next().unwrap();
        // Everything after the first dot
        let file_extension: Vec<&str> = file_name_and_extension.split('.').collect();
        let file_extension = file_extension[1..].join(".");
        // The name where the current file is located relative to the resource's folder
        let file_path = dir_entry.path().to_str().unwrap();
        let subdirectory_name = file_path.split(resources_path.as_str()).nth(1).unwrap().replace(file_name_and_extension, "");
        println!("Packing file '{}{}.{}'", subdirectory_name, file_name, file_extension);

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
            let resource_timestamp = file_metadata.modified().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

            // Did we edit the file?
            if resource_timestamp > *packed_timestamp {
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
            "vrsh.glsl" | "frsh.glsl" | "cmpt.glsl" | "func.glsl" => {
                // This is a shader
                resource = convert_shader(&file, file_extension.as_str());
            }
            "mdl3d" => {
                // This is a 3D model
                resource = convert_mdl3d(&file);
            }
            "png" => {
                // This is a texture
                resource = convert_texture(&mut file, file_path);
            }
            "font" => {
                // This is a custom font, just pass the bytes since we don't do special stuff here
                // Reader
                let mut reader = BufReader::new(file);
                let mut bytes = Vec::<u8>::new();
                reader.read_to_end(&mut bytes).unwrap();
                println!("{}", bytes.len());
                resource = Resource::Bytes(bytes);
            }
            _ => {}
        }

        // Now time to actually pack the resource
        let packed_file_path = format!("{}{}.pkg", packed_resources_path, packed_file_hashed_name);
        // Create the file
        let packed_file = File::create(packed_file_path).unwrap();
        let packed_file_metadata = packed_file.metadata().ok()?;
        let last_time_packed = packed_file_metadata.modified().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let mut writer = BufWriter::new(packed_file);
        match resource {
            Resource::Shader(shader, _) => {
                // This is a shader
                pack_shader(&mut writer, shader).unwrap();
            }
            Resource::Model(model) => {
                // This a 3D model
                pack_model(&mut writer, model).unwrap();
            }
            Resource::Texture(texture, _) => {
                // This a texture
                pack_texture(&mut writer, texture).unwrap();
            }
            Resource::Bytes(b) => {
                // Pass on the bytes
                writer.write_all(b.as_bytes()).unwrap();
            }
            _ => {}
        }

        // After packing the file, we need to append it to log file
        log_file_packed_timestamps.insert(packed_file_hashed_name, last_time_packed);
    }

    // Check if there are files in the packed hashed names timestamp hashmap that don't exist in the resource folder
    log_file_packed_timestamps.retain(|hashed, _| {
        // Only keep the hashed names that are actually exist in the original resource folder
        let keep = hashed_names.contains(hashed);
        if !keep {
            let packed_file_path = format!("{}{}.pkg", packed_resources_path, hashed);
            let packed_file_path_path = Path::new(&packed_file_path);
            println!("{}", packed_file_path);
            // Check if the file even exists first
            if packed_file_path_path.exists() {
                remove_file(packed_file_path).unwrap();
            }
        }
        keep
    });

    // Rewrite all the hashed names and timestamps that we saved in the hashmap
    for (name, timestamp) in log_file_packed_timestamps {
        log_file_writer.write_u64::<LittleEndian>(name).ok()?;
        log_file_writer.write_u64::<LittleEndian>(timestamp).ok()?;
    }

    // Packed the resources sucsessfully
    Some(())
}

pub fn pack(target_dir: String, project_dir: String) {
    println!("Target Dir: {}", target_dir);
    println!("Project Dir: {}", project_dir);
    pack_resources(project_dir.clone());
    println!("{}", format!("{}\\resources\\packed_resources", project_dir).as_str());
    println!("{}", format!("{}packed-resources\\", target_dir).as_str());
    let output = Command::new("robocopy")
        .args(&[
            format!("{}\\resources\\packed-resources", project_dir).as_str(),
            format!("{}packed-resources\\", target_dir).as_str(),
            "/mir",
            "/njh",
            "/njs",
            "/ndl",
            "/nc",
            "/ns",
        ])
        .spawn()
        .expect("Da bruh");
    /*
    let target_path: Vec<&str> = target_path.split('\\').collect();
    // The src path
    let src_path = target_path[..(target_path.len() - 5)].join("\\");
    let src_path = format!("{}\\", src_path);
    */
}
