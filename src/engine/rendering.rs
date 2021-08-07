use std::{collections::HashMap, ffi::{CString, c_void}, mem::size_of, ptr::null};
use crate::engine::core::defaults::components::components::Renderer;
use crate::engine::resources::Resource;
use crate::engine::core::world::World;
use gl;

// A window class to organize things
#[derive(Default)]
pub struct Window {
	pub fullscreen: bool,
	pub size: (i32, i32),
	pub system_renderer_component_index: u16,
}

// Shader manager
pub struct ShaderManager {
	pub shaders: HashMap<String, Shader>,
	pub subshaders: HashMap<String, SubShader>,	
}

// Default
impl Default for ShaderManager {
	fn default() -> Self {
		Self {
			shaders: HashMap::new(),
			subshaders: HashMap::new(),
		}
	}
}

impl ShaderManager {		
	// Caches a specific subshader, if it already exists then give back a reference
	pub fn cache_subshader(&mut self, subshader: SubShader) -> Option<&mut SubShader> {
		if !self.subshaders.contains_key(&subshader.name) {
			// Cache the shader for later use
			let name_clone = subshader.name.clone();
			self.subshaders.insert(name_clone.clone(), subshader);
			return self.subshaders.get_mut(&name_clone);
		} else {
			return self.subshaders.get_mut(&subshader.name);
		}
	}
	// Cached a specific shader (An actual runnable shader with uniforms and all) 
	pub fn cache_shader(&mut self, shader: Shader) -> Option<&mut Shader> {
		if !self.shaders.contains_key(&shader.name) {
			// Cache the shader for later use
			let name_clone = shader.name.clone();
			self.shaders.insert(name_clone.clone(), shader);
			return self.shaders.get_mut(&name_clone);
		} else {
			panic!("Cannot cache the same shader twice!");
			return None;
		}
	}
	// Gets a specific subshader from cache
	pub fn get_subshader(&mut self, subshader_name: &String) -> Option<&mut SubShader> {
		// Make sure it exists
		if self.subshaders.contains_key(subshader_name) {
			return self.subshaders.get_mut(subshader_name);
		} else {
			return None;
		}
	}
	// Gets a specific shader from cache
	pub fn get_shader(&self, shader_name: &String) -> Option<&Shader> {
		// Make sure it exists
		if self.shaders.contains_key(shader_name) {
			return self.shaders.get(shader_name);
		} else {
			return None;
		}
	}
}

// A shader that contains two sub shaders that are compiled independently
pub struct Shader {
	pub name: String,
	pub program: u32,
	pub finalized: bool,
	pub linked_subshaders_programs: Vec<u32>,
}

impl Default for Shader {
	fn default() -> Self {
		unsafe {
			Self {
				name: String::from("Undefined"),
				program: gl::CreateProgram(),
				finalized: false,
				linked_subshaders_programs: Vec::new()
			}
		}
	}
}

impl Shader {
	// Creates a shader from a vertex subshader file and a fragment subshader file
	pub fn from_vr_fr_subshader_files(vertex_file: &str, fragment_file: &str, world: &mut World) -> Self {
		let mut shader = Self::default();
		shader.name = format!("{}_{}", vertex_file, fragment_file);
		{
			{
				let default_vert_subshader_resource = world.resource_manager.load_resource(vertex_file, "shaders\\").unwrap();
				// Link the vertex and fragment shaders
				let mut vert_subshader = SubShader::from_resource(default_vert_subshader_resource).unwrap();
				// Compile the subshader
				vert_subshader.compile_subshader();
				// Cache it, and link it
				let vert_subshader = world.shader_manager.cache_subshader(vert_subshader).unwrap();
				shader.link_subshader(&vert_subshader);
			}
			{
				let default_frag_subshader_resource = world.resource_manager.load_resource(fragment_file, "shaders\\").unwrap();
				// Link the vertex and fragment shaders
				let mut frag_subshader = SubShader::from_resource(default_frag_subshader_resource).unwrap();
				// Compile the subshader
				frag_subshader.compile_subshader();
				// Cache it, and link it
				let frag_subshader = world.shader_manager.cache_subshader(frag_subshader).unwrap();
				shader.link_subshader(&frag_subshader);
			}
		}	
		return shader;
	}
	// Finalizes a vert/frag shader by compiling it
	pub fn finalize_shader(&mut self) {
		unsafe {
			// Finalize the shader and stuff
			gl::LinkProgram(self.program);

			// Check for errors
			// Check for any errors
			let mut info_log_length: i32 = 0;
			let info_log_length_ptr: *mut i32 = &mut info_log_length;
			let mut result: i32 = 0;		
			let result_ptr: *mut i32 = &mut result;	
			gl::GetProgramiv(self.program, gl::INFO_LOG_LENGTH, info_log_length_ptr);
			gl::GetProgramiv(self.program, gl::LINK_STATUS, result_ptr);
			// Print any errors that might've happened while finalizing this shader
			if info_log_length > 0 {
				let mut log: Vec<i8> = vec![0; info_log_length as usize + 1];
				gl::GetProgramInfoLog(self.program, info_log_length, 0 as *mut i32, log.as_mut_ptr());
				println!("Error while finalizing shader {}!:", self.name);
				let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect(); 
				let string = String::from_utf8(printable_log).unwrap();
				println!("Error: \n\x1b[31m{}", string);
				println!("\x1b[0m");
				panic!();
			}

			for subshader_program in self.linked_subshaders_programs.iter() {
				gl::DetachShader(self.program, subshader_program.clone());
			}
			self.finalized = true;
		}
	}
	// Use this shader for rendering a specific entity
	pub fn use_shader(&self) {
		// Check if the program even was finalized and ready for use
		if self.finalized {
			unsafe {
				gl::UseProgram(self.program);
			}
		} else {
			println!("Shader '{}' not finalized!", self.name);
		}
	} 
	// Link a specific subshader to this shader
	pub fn link_subshader(&mut self, subshader: &SubShader) { 
		self.linked_subshaders_programs.push(subshader.program);
		unsafe {
			gl::AttachShader(self.program, subshader.program);
		}
	}
}

// Impl block for interfacing with the OpenGL shader, like setting uniforms and scuh 
impl Shader {
	// Get the location of a specific uniform, using it's name
	pub fn get_uniform_location(&self, name: &str) -> i32 {
		unsafe {
			return gl::GetUniformLocation(self.program, CString::new(name).unwrap().as_ptr());
		}
	}
	// Set a scalar uniform
	pub fn set_scalar_1_uniform(&self, location: i32, value: f32) {
		unsafe {
			gl::Uniform1f(location, value);
		}
	}
	// Set a scalar x2 uniform
	pub fn set_scalar_2_uniform(&self, location: i32, values: (f32, f32)) {
		unsafe {
			gl::Uniform2f(location, values.0, values.1);
		}
	}
	// Set a scalar x3 uniform
	pub fn set_scalar_3_uniform(&self, location: i32, values: (f32, f32, f32)) {
		unsafe {
			gl::Uniform3f(location, values.0, values.1, values.2);
		}
	}
	// Set a matrix 4x4
	pub fn set_matrix_44_uniform(&self, location: i32, matrix: glam::Mat4) {
		unsafe {
			let ptr: *const f32 = &matrix.as_ref()[0];
			gl::UniformMatrix4fv(location, 1, gl::FALSE, ptr);
		}
	}
	// Set a texture basically
	pub fn set_texture2d(&self, location: i32, texture_id: u32, active_texture_id: u32) {
		unsafe {
			gl::ActiveTexture(active_texture_id);
			gl::BindTexture(gl::TEXTURE_2D, texture_id);
			gl::Uniform1i(location, active_texture_id as i32 - 33984);
		}
	}
}

// Sub shader type
#[derive(Debug, Clone)]
pub enum SubShaderType {
	Vertex,
	Fragment,
	Geometry,
}

// A sub shader, could be a geometry, vertex, or fragment shader
#[derive(Clone)]
pub struct SubShader {
	pub program: u32,
	pub name: String,
	pub source: String,
	pub subshader_type: SubShaderType,
}

impl SubShader {
	// Create a subshader from a loaded subshader resource
	pub fn from_resource(resource: &Resource) -> Option<Self> {
		match resource {    		
    		Resource::Shader(shader) => {
				// Turn the loaded sub shader into a normal sub shader
				let subshader = Self {
					name: shader.name.clone(),
        			program: 0,
        			source: shader.source.clone(),
        			subshader_type: shader.subshader_type.clone(),
    			};
				return Some(subshader);
			},
    		_ => return None,
		}
	}
	// Compile the current subshader's source code
	pub fn compile_subshader(&mut self) {
		let mut shader_type: u32 = 0;
		match self.subshader_type {
    		SubShaderType::Vertex => shader_type = gl::VERTEX_SHADER,
    		SubShaderType::Fragment => shader_type = gl::FRAGMENT_SHADER,
    		SubShaderType::Geometry => shader_type = gl::GEOMETRY_SHADER,
		}
		unsafe {
			self.program = gl::CreateShader(shader_type);
			// Compile the shader
			let cstring = CString::new(self.source.clone()).unwrap();
			let shader_source: *const i8 = cstring.as_ptr();
			gl::ShaderSource(self.program, 1, &shader_source, null());
			gl::CompileShader(self.program);

			// Check for any errors
			let mut info_log_length: i32 = 0;
			let info_log_length_ptr: *mut i32 = &mut info_log_length;
			let mut result: i32 = 0;		
			let result_ptr: *mut i32 = &mut result;	
			gl::GetShaderiv(self.program, gl::INFO_LOG_LENGTH, info_log_length_ptr);
			gl::GetShaderiv(self.program, gl::LINK_STATUS, result_ptr);
			// Print any errors that might've happened while compiling this subshader
			if info_log_length > 0 {
				let mut log: Vec<i8> = vec![0; info_log_length as usize + 1];
				gl::GetShaderInfoLog(self.program, info_log_length, 0 as *mut i32, log.as_mut_ptr());
				println!("Error while compiling sub-shader {}!:", self.name);
				let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect(); 
				let string = String::from_utf8(printable_log).unwrap();
				println!("Error: \n\x1b[31m{}", string);
				println!("\x1b[0m");
				panic!();
			}

			println!("\x1b[32mSubshader {} compiled succsessfully!\x1b[0m", self.name);
		}
	}	
}

// A simple model that holds vertex, normal, and color data
#[derive(Default, Debug)]
pub struct Model {
	pub vertices: Vec<glam::Vec3>,
	pub normals: Vec<glam::Vec3>,
	pub tangents: Vec<glam::Vec3>,
	pub uvs: Vec<glam::Vec2>,
	pub triangles: Vec<u32>,
}

impl Model {
	// Turns a loaded resource model into an actual model
	pub fn from_resource(resource: &Resource) -> Option<Self> {
		match resource {    		
    		Resource::Model(model) => {
				// Turn the loaded model into a normal model
				let new_model = Self {
        			vertices: model.vertices.clone(),
					normals: model.normals.clone(),
					tangents: model.tangents.clone(),
					uvs: model.uvs.clone(),
        			triangles: model.indices.clone(),
   				};
				return Some(new_model);
			},
    		_ => return None,
		}
	}
	// Flip all the triangles in the mesh, basically making it look inside out. This also flips the normals
	pub fn flip_triangles(&mut self) {
		for i in (0..self.triangles.len()).step_by(3) {
			// Swap the first and last index of each triangle
			let copy = self.triangles[i];
			self.triangles[i] = self.triangles[i + 2];
			self.triangles[i + 2] = copy;			
		}
	}
}

// A texture manager
#[derive(Default)]
pub struct TextureManager {
	pub texture_ids: HashMap<String, u16>,
	pub cached_textures: Vec<Texture>,
}

impl TextureManager {
	// Get a reference to a texture from the texture manager's cache
	pub fn get_texture(&self, id: i16) -> &Texture {
		return self.cached_textures.get(id as usize).unwrap();
	}
	// Get a mutable reference to a texture from the texture manager's cache
	pub fn get_texture_mut(&mut self, id: i16) -> &mut Texture {
		return self.cached_textures.get_mut(id as usize).unwrap();
	}
	// Add a texture to the manager
	pub fn cache_texture(&mut self, texture: Texture) -> i16 {
		let name_clone = texture.name.clone();
		// Make sure the texture isn't cached already
		if !self.texture_ids.contains_key(&name_clone) {
			self.cached_textures.push(texture);
			let texture_id = self.cached_textures.len() as i16 - 1;
			println!("Cache texture: '{}' with texture id '{}'", &name_clone, &texture_id);
			self.texture_ids.insert(name_clone, texture_id as u16);
			return texture_id;
		} else {
			return self.texture_ids.get(&name_clone).unwrap().clone() as i16;
		}

	}
	// Get the texture id of a specific texture using it's name
	pub fn get_texture_id(&self, name: &str) -> i16 {
		// Check if the texture even exists
		if self.texture_ids.contains_key(&name.to_string()) {
			return self.texture_ids.get(name).unwrap().clone() as i16;
		} else {
			panic!("Texture was not cached!");
		}
	}
}

// A texture
#[derive(Default)]
pub struct Texture {
	pub width: u16,
	pub height: u16,
	pub id: u32,
	pub name: String,
	pub internal_format: u32,
	pub format: u32,
	pub data_type: u32,
}

impl Texture {
	// Loads a texture and caches it, then returns the texture id
	pub fn load_from_file(file: &str, world: &mut World) -> Option<i16> {
		let texture_resource = world.resource_manager.load_resource(file, "textures\\")?;
		let texture = Texture::from_resource(texture_resource)?;
		let id = world.texture_manager.cache_texture(texture);
		return Some(id);
	}
	// Convert the resource to a texture
    pub fn from_resource(resource: &Resource) -> Option<Self> {
		match resource {
			Resource::Texture(texture) => {
				let width = texture.width;
				let height = texture.height;
				let mut new_texture = Self::create_rgb_texture(texture.name.clone(), width, height, &texture.raw_pixels);
				new_texture.name = texture.name.clone();
				return Some(new_texture);
			}
			_ => { return None }
		}
    }
	// Creates a new empty texture from a specified size
	pub fn create_new_texture(width: u16, height: u16, internal_format: u32, format: u32, data_type: u32) -> Self {
		let mut texture = Self {
			width,
			height,
			id: 0,
			internal_format,
			name: String::from("Untitled"),
			format,
			data_type,
		};

		// Create the OpenGL texture and set it's data to null since it's empty
		unsafe {
			gl::GenTextures(1, &mut texture.id as *mut u32);
			gl::BindTexture(gl::TEXTURE_2D, texture.id);
			gl::TexImage2D(gl::TEXTURE_2D, 0, texture.internal_format as i32, width as i32, height as i32, 0, texture.format, texture.data_type, null());
		
			// Mag and min filters
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
		}

		return texture;
	}
	// Creates a rgb texture from a vector filled with bytes
	pub fn create_rgb_texture(name: String, width: u16, height: u16, pixels: &Vec<u8>) -> Self {
		let mut texture = Self {
			width,
			height,
			id: 0,
			internal_format: gl::RGB,
			name,
			format: gl::RGB,
			data_type: gl::UNSIGNED_BYTE,
		};
		// Create the OpenGL texture and set it's data to null since it's empty
		unsafe {
			gl::GenTextures(1, &mut texture.id as *mut u32);
			gl::BindTexture(gl::TEXTURE_2D, texture.id);
			gl::TexImage2D(gl::TEXTURE_2D,
				0,
				texture.internal_format as i32,
				width as i32,
				height as i32,
				0,				
				texture.format,
				texture.data_type,
				pixels.as_ptr() as *const c_void);
			// Mag and min filters
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
		}

		return texture;
	}
	// Update the size of this specific texture
	pub fn update_size(&mut self, xsize: u32, ysize: u32) {
		unsafe {
			gl::BindTexture(gl::TEXTURE_2D, self.id);
			gl::TexImage2D(gl::TEXTURE_2D, 0, self.internal_format as i32, xsize as i32, ysize as i32, 0, self.format, self.data_type, null());
		}
	} 
}

// The current render state of the entity
pub enum EntityRenderState {
	Visible,
	Invisible,
}

impl Default for EntityRenderState {
	fn default() -> Self { Self::Visible }
}

// Struct that hold the model's information from OpenGL
#[derive(Default)]
pub struct ModelDataGPU {
	pub vertex_buf: u32,
	pub normal_buf: u32,
	pub uv_buf: u32,
	pub tangent_buf: u32,
	pub vertex_array_object: u32,
	pub element_buffer_object: u32,
	pub initialized: bool,
	pub model_matrix: glam::Mat4,
}

impl Renderer {
	// Updates the model matrix using a position and a rotation
	pub fn update_model_matrix(&mut self, position: glam::Vec3, rotation: glam::Quat, scale: f32) {
		let model_matrix = glam::Mat4::from_quat(rotation) * glam::Mat4::from_translation(position) * glam::Mat4::from_scale(glam::vec3(scale, scale, scale));
		self.gpu_data.model_matrix = model_matrix;
	}
	// When we update the model and want to refresh it's OpenGL data
	pub fn refresh_model(&mut self) {
		unsafe {
			// Create the VAO
			gl::GenVertexArrays(1, &mut self.gpu_data.vertex_array_object);
			gl::BindVertexArray(self.gpu_data.vertex_array_object);

			// Create the EBO
			gl::GenBuffers(1, &mut self.gpu_data.element_buffer_object);
			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.gpu_data.element_buffer_object);
			gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (self.model.triangles.len() * size_of::<u32>()) as isize, self.model.triangles.as_ptr() as *const c_void, gl::STATIC_DRAW);

			// Create the vertex buffer and populate it
			gl::GenBuffers(1, &mut self.gpu_data.vertex_buf);
			gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.vertex_buf);
			gl::BufferData(gl::ARRAY_BUFFER, (self.model.vertices.len() * size_of::<f32>() * 3) as isize, self.model.vertices.as_ptr() as *const c_void, gl::STATIC_DRAW);

			// Create the normals buffer
			gl::GenBuffers(1, &mut self.gpu_data.normal_buf);
			gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.normal_buf);
			gl::BufferData(gl::ARRAY_BUFFER, (self.model.normals.len() * size_of::<f32>() * 3) as isize, self.model.normals.as_ptr() as *const c_void, gl::STATIC_DRAW);

			// And it's brother, the tangent buffer			
			gl::GenBuffers(1, &mut self.gpu_data.tangent_buf);
			gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.tangent_buf);
			gl::BufferData(gl::ARRAY_BUFFER, (self.model.tangents.len() * size_of::<f32>() * 3) as isize, self.model.tangents.as_ptr() as *const c_void, gl::STATIC_DRAW);

			// Finally, the texture coordinates buffer
			gl::GenBuffers(1, &mut self.gpu_data.uv_buf);
			gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.uv_buf);
			gl::BufferData(gl::ARRAY_BUFFER, (self.model.uvs.len() * size_of::<f32>() * 2) as isize, self.model.uvs.as_ptr() as *const c_void, gl::STATIC_DRAW);


			// Create the vertex attrib arrays
			gl::EnableVertexAttribArray(0);
			gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.vertex_buf);
			gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());	

			// Normal attribute
			gl::EnableVertexAttribArray(1);
			gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.normal_buf);
			gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, null());		

			// Tangent attribute
			gl::EnableVertexAttribArray(2);
			gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.tangent_buf);
			gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, 0, null());	

			// UV attribute
			gl::EnableVertexAttribArray(3);
			gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.uv_buf);
			gl::VertexAttribPointer(3, 2, gl::FLOAT, gl::FALSE, 0, null());	

			self.gpu_data.initialized = true;
			println!("Initialized model with '{}' vertices and '{}' triangles", self.model.vertices.len(), self.model.triangles.len());
			// Unbind
			gl::BindVertexArray(0);
			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
		}
	}

	// Dispose of our model
	pub fn dispose_model(&mut self) {
		unsafe {
			// Delete the vertex array
			gl::DeleteBuffers(1, &mut self.gpu_data.vertex_buf);
		}
	}
}