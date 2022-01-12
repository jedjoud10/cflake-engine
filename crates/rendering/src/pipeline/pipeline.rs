use crate::{
    compute::{ComputeShader, ComputeShaderExecutionSettings},
    object::{ObjectBuildingTask, ObjectID, PipelineTask, PipelineTaskStatus, TaskID},
    pipec,
    pipeline::{camera::Camera, sender},
    Material, Model, ModelBuffers, PipelineRenderer, Renderer, Shader, ShaderSettings, ShaderSource, ShaderSourceType, ShaderUniformsSettings, Texture, TextureFilter,
    TextureFlags, TextureType, TextureWrapping, Window,
};
use glfw::Context;
use ordered_vec::shareable::ShareableOrderedVec;
use std::{
    collections::HashSet,
    ffi::{c_void, CString},
    mem::size_of,
    ptr::null,
    sync::{atomic::AtomicPtr, mpsc::Sender, Arc, Barrier, RwLock},
};

// Some default values like the default material or even the default shader
pub(crate) struct DefaultPipelineObjects {
    pub(crate) diffuse_tex: ObjectID<Texture>,
    pub(crate) normals_tex: ObjectID<Texture>,
    pub(crate) shader: ObjectID<Shader>,
    pub(crate) material: ObjectID<Material>,
    pub(crate) model: ObjectID<Model>,
}

// The rendering pipeline. It can be shared around using Arc, but we are only allowed to modify it on the Render Thread
// This is only updated at the end of each frame, so we don't have to worry about reading it from multiple threads since no one will be writing to it at that times
#[derive(Default)]
pub struct Pipeline {
    // We will buffer the tasks, so that way whenever we receive a task internally from the Render Thread itself we can just wait until we manually flush the tasks to execute all at once
    tasks: Vec<(PipelineTask, TaskID)>,
    // We store the Pipeline Objects, for each Pipeline Object type
    // We will create these Pipeline Objects *after* they have been created by OpenGL (if applicable)
    pub(crate) materials: ShareableOrderedVec<Material>,
    pub(crate) models: ShareableOrderedVec<(Model, ModelBuffers)>,
    pub(crate) renderers: ShareableOrderedVec<Renderer>,
    pub(crate) shaders: ShareableOrderedVec<Shader>,
    pub(crate) compute_shaders: ShareableOrderedVec<ComputeShader>,
    pub(crate) textures: ShareableOrderedVec<Texture>,

    // Store a struct that is filled with default values that we initiate at the start of the creation of this pipeline
    pub(crate) defaults: Option<DefaultPipelineObjects>,

    // Store the status for all of our tasks
    pub(crate) task_statuses: ShareableOrderedVec<PipelineTaskStatus>,
    // Store the TaskIDs for tasks that have finished execution from last frame
    pub(crate) last_frame_task_statuses: HashSet<usize>,
    // The current main camera that is rendering the world
    pub(crate) camera: Camera,
    // Our window
    pub window: Window,

    // Should we quit from the render thread?
    should_quit: bool,
}

impl Pipeline {
    // Set the buffered tasks from RX messages
    pub fn add_tasks(&mut self, messages: Vec<(PipelineTask, TaskID)>) {
        messages.iter().for_each(|(_, id)| {
            self.task_statuses.insert(id.index, PipelineTaskStatus::Pending);
        });
        self.tasks.extend(messages);
    }
    // Flush all the buffered tasks, and execute them
    // We should do this at the end of each frame, but we can force execution of it if we are running it internally
    pub fn flush(&mut self) {
        // We must take the commands first
        let tasks = std::mem::take(&mut self.tasks);
        // Clear
        self.last_frame_task_statuses.clear();
        for (task, task_id) in tasks {
            // Now we must execute these tasks
            match task {
                // Creation tasks
                PipelineTask::CreateTexture(t) => self.texture_create(t),
                PipelineTask::CreateMaterial(t) => self.material_create(t),
                PipelineTask::CreateShader(t) => self.shader_create(t),
                PipelineTask::CreateModel(t) => self.model_create(t),
                PipelineTask::CreateRenderer(t) => self.renderer_create(t),
                PipelineTask::CreateComputeShader(t) => self.compute_create(t),

                PipelineTask::RunComputeShader(id, settings) => self.compute_run(id, settings),
                PipelineTask::UpdateRendererMatrix(id, matrix) => self.renderer_update_matrix(id, matrix),

                // Others
                PipelineTask::Quit => self.should_quit = false,
            }

            // After executing the tasks, we must remove our current status, and add the index to the valid task statuses
            let mut status = self.task_statuses.remove(task_id.index).unwrap();
            status = PipelineTaskStatus::Finished;
            self.last_frame_task_statuses.insert(task_id.index);
        }
    }

    // Get a material using it's respective ID
    pub fn get_material(&self, id: ObjectID<Material>) -> Option<&Material> {
        if let Some(index) = id.index {
            self.materials.get(index)
        } else {
            None
        }
    }
    // Get a model using it's respective ID
    pub fn get_model(&self, id: ObjectID<Model>) -> Option<&(Model, ModelBuffers)> {
        if let Some(index) = id.index {
            self.models.get(index)
        } else {
            None
        }
    }
    // Get a renderer using it's respective ID
    pub fn get_renderer(&self, id: ObjectID<Renderer>) -> Option<&Renderer> {
        if let Some(index) = id.index {
            self.renderers.get(index)
        } else {
            None
        }
    }
    // Get a shader using it's respective ID
    pub fn get_shader(&self, id: ObjectID<Shader>) -> Option<&Shader> {
        if let Some(index) = id.index {
            self.shaders.get(index)
        } else {
            None
        }
    }
    // Get a compute shader using it's respective ID
    pub fn get_compute_shader(&self, id: ObjectID<ComputeShader>) -> Option<&ComputeShader> {
        if let Some(index) = id.index {
            self.compute_shaders.get(index)
        } else {
            None
        }
    }
    // Get a texture using it's texture
    pub fn get_texture(&self, id: ObjectID<Texture>) -> Option<&Texture> {
        if let Some(index) = id.index {
            self.textures.get(index)
        } else {
            None
        }
    }

    // Actually update our data
    // Add the renderer
    pub fn renderer_create(&mut self, task: ObjectBuildingTask<Renderer>) {
        // Get the renderer data, if it does not exist then use the default renderer data
        let renderer = task.0;
        let defaults = self.defaults.as_ref().unwrap();
        let material_id = self.get_material(defaults.material);
        let model_id = self.get_model(defaults.model);

        self.renderers.insert(task.1.index.unwrap(), renderer);
    }
    // Remove the renderer using it's renderer ID
    pub fn renderer_dispose(&mut self, id: ObjectID<Renderer>) {
        self.renderers.remove(id.index.unwrap());
    }
    // Update a renderer's matrix
    pub fn renderer_update_matrix(&mut self, id: ObjectID<Renderer>, matrix: veclib::Matrix4x4<f32>) {}
    // Create a shader and cache it. We do not cache the "subshader" though
    pub fn shader_create(&mut self, task: ObjectBuildingTask<Shader>) {
        // Compile a single shader source
        fn compile_single_source(source_data: ShaderSource) -> u32 {
            let shader_type: u32;
            println!("\x1b[33mCompiling & Creating Shader Source {}...\x1b[0m", source_data.path);
            match source_data._type {
                ShaderSourceType::Vertex => shader_type = gl::VERTEX_SHADER,
                ShaderSourceType::Fragment => shader_type = gl::FRAGMENT_SHADER,
                ShaderSourceType::Compute => {
                    panic!()
                } // We are not allowed to create compute shaders using the normal create_shader function
            }
            unsafe {
                let program = gl::CreateShader(shader_type);
                // Compile the shader
                let cstring = CString::new(source_data.text.clone()).unwrap();
                let shader_source: *const i8 = cstring.as_ptr();
                gl::ShaderSource(program, 1, &shader_source, null());
                gl::CompileShader(program);
                // Check for any errors
                let mut info_log_length: i32 = 0;
                let info_log_length_ptr: *mut i32 = &mut info_log_length;
                gl::GetShaderiv(program, gl::INFO_LOG_LENGTH, info_log_length_ptr);
                // Print any errors that might've happened while compiling this shader source
                if info_log_length > 0 {
                    let mut log: Vec<i8> = vec![0; info_log_length as usize + 1];
                    gl::GetShaderInfoLog(program, info_log_length, std::ptr::null_mut::<i32>(), log.as_mut_ptr());
                    println!("Error while compiling shader source {}!:", source_data.path);
                    let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect();
                    let string = String::from_utf8(printable_log).unwrap();

                    println!("Error: \n\x1b[31m{}", string);
                    println!("\x1b[0m");
                    panic!();
                }

                println!("\x1b[32mShader Source {} compiled succsessfully!\x1b[0m", source_data.path);
                program
            }
        }
        // Extract the shader
        let mut shader = task.0;
        let shader_name = shader.sources.iter().map(|(name, _)| name.clone()).collect::<Vec<String>>().join("_");

        // Actually compile the shader now
        println!("\x1b[33mCompiling & Creating Shader {}...\x1b[0m", shader_name);
        unsafe {
            let program = gl::CreateProgram();

            // Create & compile the shader sources and link them
            let taken = std::mem::take(&mut shader.sources);
            let programs: Vec<u32> = taken.into_iter().map(|(path, data)| compile_single_source(data)).collect::<Vec<_>>();
            // Link
            for shader in programs.iter() {
                gl::AttachShader(program, *shader)
            }

            // Finalize the shader and stuff
            gl::LinkProgram(program);

            // Check for any errors
            let mut info_log_length: i32 = 0;
            let info_log_length_ptr: *mut i32 = &mut info_log_length;
            let mut result: i32 = 0;
            let result_ptr: *mut i32 = &mut result;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, info_log_length_ptr);
            gl::GetProgramiv(program, gl::LINK_STATUS, result_ptr);
            // Print any errors that might've happened while finalizing this shader
            if info_log_length > 0 {
                let mut log: Vec<i8> = vec![0; info_log_length as usize + 1];
                gl::GetProgramInfoLog(program, info_log_length, std::ptr::null_mut::<i32>(), log.as_mut_ptr());
                println!("Error while finalizing shader {}!:", shader_name);
                let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect();
                let string = String::from_utf8(printable_log).unwrap();
                println!("Error: \n\x1b[31m{}", string);
                println!("\x1b[0m");
                panic!();
            }
            // Detach shaders
            for shader in programs.iter() {
                gl::DetachShader(program, *shader);
            }
            println!("\x1b[32mShader {} compiled and created succsessfully!\x1b[0m", shader_name);
        }
        // Add the shader at the end
        self.shaders.insert(task.1.index.unwrap(), shader);
    }
    // Create a compute shader and cache it
    pub fn compute_create(&mut self, task: ObjectBuildingTask<ComputeShader>) {
        // Extract the shader
        let shader = task.0;

        // Actually compile the shader now
        println!("\x1b[33mCompiling & Creating Compute Shader {}...\x1b[0m", shader.source.path);
        println!("\x1b[33mCompiling & Creating Compute Shader Source {}...\x1b[0m", shader.source.path);
        let shader_source_program = unsafe {
            // Compiling the source
            let program = gl::CreateShader(gl::COMPUTE_SHADER);
            // Compile the shader
            let cstring = CString::new(shader.source.text.clone()).unwrap();
            let shader_source: *const i8 = cstring.as_ptr();
            gl::ShaderSource(program, 1, &shader_source, null());
            gl::CompileShader(program);
            // Check for any errors
            let mut info_log_length: i32 = 0;
            let info_log_length_ptr: *mut i32 = &mut info_log_length;
            gl::GetShaderiv(program, gl::INFO_LOG_LENGTH, info_log_length_ptr);
            // Print any errors that might've happened while compiling this shader source
            if info_log_length > 0 {
                let mut log: Vec<i8> = vec![0; info_log_length as usize + 1];
                gl::GetShaderInfoLog(program, info_log_length, std::ptr::null_mut::<i32>(), log.as_mut_ptr());
                println!("Error while compiling shader source {}!:", shader.source.path);
                let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect();
                let string = String::from_utf8(printable_log).unwrap();

                println!("Error: \n\x1b[31m{}", string);
                println!("\x1b[0m");
                panic!();
            }

            println!("\x1b[32mSubshader {} compiled succsessfully!\x1b[0m", shader.source.path);
            program
        };
        unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, shader_source_program);
            // Finalize the shader and stuff
            gl::LinkProgram(program);

            // Check for any errors
            let mut info_log_length: i32 = 0;
            let info_log_length_ptr: *mut i32 = &mut info_log_length;
            let mut result: i32 = 0;
            let result_ptr: *mut i32 = &mut result;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, info_log_length_ptr);
            gl::GetProgramiv(program, gl::LINK_STATUS, result_ptr);
            // Print any errors that might've happened while finalizing this shader
            if info_log_length > 0 {
                let mut log: Vec<i8> = vec![0; info_log_length as usize + 1];
                gl::GetProgramInfoLog(program, info_log_length, std::ptr::null_mut::<i32>(), log.as_mut_ptr());
                println!("Error while finalizing shader {}!:", shader.source.path);
                let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect();
                let string = String::from_utf8(printable_log).unwrap();
                println!("Error: \n\x1b[31m{}", string);
                println!("\x1b[0m");
                panic!();
            }
            // Detach shader source
            gl::DetachShader(program, shader_source_program);
            println!("\x1b[32mShader {} compiled and created succsessfully!\x1b[0m", shader.source.path);
        }
        // Add the shader at the end
        self.compute_shaders.insert(task.1.index.unwrap(), shader);
    }
    // Create a model
    pub fn model_create(&mut self, task: ObjectBuildingTask<Model>) {
        let mut model = task.0;
        let mut buffers = ModelBuffers::default();
        buffers.triangle_count = model.triangles.len();
        unsafe {
            // Create the VAO
            gl::GenVertexArrays(1, &mut buffers.vertex_array_object);
            gl::BindVertexArray(buffers.vertex_array_object);

            // Create the EBO
            gl::GenBuffers(1, &mut buffers.element_buffer_object);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffers.element_buffer_object);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (model.triangles.len() * size_of::<u32>()) as isize,
                model.triangles.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Create the vertex buffer and populate it
            gl::GenBuffers(1, &mut buffers.vertex_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffers.vertex_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (model.vertices.len() * size_of::<f32>() * 3) as isize,
                model.vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Create the normals buffer
            gl::GenBuffers(1, &mut buffers.normal_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffers.normal_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (model.normals.len() * size_of::<f32>() * 3) as isize,
                model.normals.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            if !model.tangents.is_empty() {
                // And it's brother, the tangent buffer
                gl::GenBuffers(1, &mut buffers.tangent_buf);
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers.tangent_buf);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (model.tangents.len() * size_of::<f32>() * 4) as isize,
                    model.tangents.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );
            }

            if !model.uvs.is_empty() {
                // The texture coordinates buffer
                gl::GenBuffers(1, &mut buffers.uv_buf);
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers.uv_buf);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (model.uvs.len() * size_of::<f32>() * 2) as isize,
                    model.uvs.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );
            }

            if !model.colors.is_empty() {
                // Finally, the vertex colors buffer
                gl::GenBuffers(1, &mut buffers.color_buf);
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers.color_buf);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (model.colors.len() * size_of::<f32>() * 3) as isize,
                    model.colors.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );
            }
            // Create the vertex attrib arrays
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffers.vertex_buf);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());

            // Normal attribute
            gl::EnableVertexAttribArray(1);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffers.normal_buf);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, null());

            if !model.tangents.is_empty() {
                // Tangent attribute
                gl::EnableVertexAttribArray(2);
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers.tangent_buf);
                gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, 0, null());
            }
            if !model.uvs.is_empty() {
                // UV attribute
                gl::EnableVertexAttribArray(3);
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers.uv_buf);
                gl::VertexAttribPointer(3, 2, gl::FLOAT, gl::FALSE, 0, null());
            }
            if !model.colors.is_empty() {
                // Vertex color attribute
                gl::EnableVertexAttribArray(4);
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers.color_buf);
            }
            gl::VertexAttribPointer(4, 3, gl::FLOAT, gl::FALSE, 0, null());
            // Unbind
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        // Add the model normally and also add it's corresponding buffers
        self.models.insert(task.1.index.unwrap(), (model, buffers));
    }
    // Dispose of a model, also remove it from the pipeline
    pub fn model_dispose(&mut self, id: ObjectID<Model>) {
        // Remove the model and it's buffers
        let (model, mut buffers) = self.models.remove(id.index.unwrap()).unwrap();
        unsafe {
            // Delete the VBOs
            gl::DeleteBuffers(1, &mut buffers.vertex_buf);
            gl::DeleteBuffers(1, &mut buffers.normal_buf);
            gl::DeleteBuffers(1, &mut buffers.uv_buf);
            gl::DeleteBuffers(1, &mut buffers.tangent_buf);
            gl::DeleteBuffers(1, &mut buffers.color_buf);
            gl::DeleteBuffers(1, &mut buffers.element_buffer_object);

            // Delete the vertex array
            gl::DeleteVertexArrays(1, &mut buffers.vertex_array_object);
        }
    }
    // Create a texture
    pub fn texture_create(&mut self, task: ObjectBuildingTask<Texture>) {
        let texture = task.0;
        let mut pointer: *const c_void = null();
        if !texture.bytes.is_empty() {
            pointer = texture.bytes.as_ptr() as *const c_void;
        }
        let ifd = crate::get_ifd(texture._format, texture._type);

        // Get the tex_type based on the TextureDimensionType
        let tex_type = match texture.ttype {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::Texture2DArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };

        let mut id: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut id as *mut u32);
            gl::BindTexture(tex_type, id);
            match texture.ttype {
                TextureType::Texture1D(width) => {
                    gl::TexImage1D(tex_type, 0, ifd.0, width as i32, 0, ifd.1, ifd.2, pointer);
                }
                // This is a 2D texture
                TextureType::Texture2D(width, height) => {
                    gl::TexImage2D(tex_type, 0, ifd.0, width as i32, height as i32, 0, ifd.1, ifd.2, pointer);
                }
                // This is a 3D texture
                TextureType::Texture3D(width, height, depth) => {
                    gl::TexImage3D(tex_type, 0, ifd.0, width as i32, height as i32, depth as i32, 0, ifd.1, ifd.2, pointer);
                }
                // This is a texture array
                TextureType::Texture2DArray(width, height, depth) => {
                    gl::TexStorage3D(
                        tex_type,
                        Texture::guess_mipmap_levels(width.max(height) as usize) as i32,
                        ifd.0 as u32,
                        width as i32,
                        height as i32,
                        depth as i32,
                    );
                    // We might want to do mipmap
                    for i in 0..depth {
                        let localized_bytes = texture.bytes[(i as usize * height as usize * 4 * width as usize)..texture.bytes.len()].as_ptr() as *const c_void;
                        gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0, i as i32, width as i32, height as i32, 1, ifd.1, ifd.2, localized_bytes);
                    }
                }
            }
            // Set the texture parameters for a normal texture
            match texture.filter {
                TextureFilter::Linear => {
                    // 'Linear' filter
                    gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                    gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                }
                TextureFilter::Nearest => {
                    // 'Nearest' filter
                    gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                    gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                }
            }
        }

        // The texture is already bound to the TEXTURE_2D
        if texture.flags.contains(TextureFlags::MIPMAPS) {
            // Create the mipmaps
            unsafe {
                gl::GenerateMipmap(tex_type);
                // Set the texture parameters for a mipmapped texture
                match texture.filter {
                    TextureFilter::Linear => {
                        // 'Linear' filter
                        gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
                        gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                    }
                    TextureFilter::Nearest => {
                        // 'Nearest' filter
                        gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_NEAREST as i32);
                        gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                    }
                }
            }
        }

        // Set the wrap mode for the texture (Mipmapped or not)
        let wrapping_mode: u32;
        match texture.wrap_mode {
            TextureWrapping::ClampToEdge => wrapping_mode = gl::CLAMP_TO_EDGE,
            TextureWrapping::ClampToBorder => wrapping_mode = gl::CLAMP_TO_BORDER,
            TextureWrapping::Repeat => wrapping_mode = gl::REPEAT,
            TextureWrapping::MirroredRepeat => wrapping_mode = gl::MIRRORED_REPEAT,
        }
        unsafe {
            // Now set the actual wrapping mode in the opengl texture
            gl::TexParameteri(tex_type, gl::TEXTURE_WRAP_S, wrapping_mode as i32);
            gl::TexParameteri(tex_type, gl::TEXTURE_WRAP_T, wrapping_mode as i32);
        }
        // Add the texture
        self.textures.insert(task.1.index.unwrap(), texture);
    }
    // Update the size of a texture
    pub fn texture_update_size(&mut self, id: ObjectID<Texture>, tt: TextureType) {
        // Get the GPU texture object
        let texture = self.get_texture(id).unwrap();
        // Check if the current dimension type matches up with the new one
        let ifd = texture.ifd;
        // This is a normal texture getting resized
        unsafe {
            match tt {
                TextureType::Texture1D(width) => {
                    gl::BindTexture(gl::TEXTURE_1D, texture.oid);
                    gl::TexImage1D(gl::TEXTURE_2D, 0, ifd.0, width as i32, 0, ifd.1, ifd.2, null());
                }
                TextureType::Texture2D(width, height) => {
                    gl::BindTexture(gl::TEXTURE_2D, texture.oid);
                    gl::TexImage2D(gl::TEXTURE_2D, 0, ifd.0, width as i32, height as i32, 0, ifd.1, ifd.2, null());
                }
                TextureType::Texture3D(width, height, depth) => {
                    gl::BindTexture(gl::TEXTURE_3D, texture.oid);
                    gl::TexImage3D(gl::TEXTURE_3D, 0, ifd.0, width as i32, height as i32, depth as i32, 0, ifd.1, ifd.2, null());
                }
                TextureType::Texture2DArray(_, _, _) => todo!(),
            }
        }
    }
    // Run a compute shader
    pub fn compute_run(&mut self, id: ObjectID<ComputeShader>, settings: ComputeShaderExecutionSettings) {
        // Execute some shader uniforms if we want to
        let group = settings.uniforms;
        if let Some(group) = group {
            // Create some shader uniforms settings that we can use
            let settings = ShaderUniformsSettings::new_compute(id);
            group.execute(self, settings).unwrap();
        }
        // Dispatch the compute shader for execution
        let axii = settings.axii;
        unsafe {
            gl::DispatchCompute(axii.0 as u32, axii.1 as u32, axii.2 as u32);
            // Force the execution and result of the compute shader. THIS IS NOT IDEAL
            gl::Finish();
        }
        // Run the tasks
        for task in settings.tasks {
            task.execute(self);
        }
    }
    // Create a materail
    pub fn material_create(&mut self, task: ObjectBuildingTask<Material>) {
        // Just add the material internally
        self.materials.insert(task.1.index.unwrap(), task.0);
    }
}

// Data that will be sent back to the main thread after we start the pipeline thread
pub struct PipelineStartData {
    // The thread handle for the render thread, so we can join it to the main thread at any time
    pub handle: std::thread::JoinHandle<()>,
    // A barrier that we can use to sync up with the main thread at the start of each frame
    pub sbarrier: Arc<Barrier>,
    // A barrier that we can use to sync up with the main thread at the end of each frame
    pub ebarrier: Arc<Barrier>,
}
// Load some defaults
fn load_defaults(pipeline: &Pipeline) -> DefaultPipelineObjects {
    use crate::texture::{TextureFilter, TextureType};
    use assets::assetc::load;

    // Create the default missing texture
    let missing = pipec::construct(load("defaults\\textures\\missing_texture.png", Texture::default().enable_mipmaps()).unwrap(), pipeline);

    // Create the default white texture
    let white = pipec::construct(
        Texture::default()
            .set_dimensions(TextureType::Texture2D(1, 1))
            .set_filter(TextureFilter::Linear)
            .set_bytes(vec![255, 255, 255, 255])
            .enable_mipmaps(),
        pipeline,
    );

    // Create the default black texture
    let black = pipec::construct(
        Texture::default()
            .set_dimensions(TextureType::Texture2D(1, 1))
            .set_filter(TextureFilter::Linear)
            .set_bytes(vec![0, 0, 0, 255])
            .enable_mipmaps(),
        pipeline,
    );

    // Create the default normal map texture
    let normals = pipec::construct(
        Texture::default()
            .set_dimensions(TextureType::Texture2D(1, 1))
            .set_filter(TextureFilter::Linear)
            .set_bytes(vec![127, 128, 255, 255])
            .enable_mipmaps(),
        pipeline,
    );

    // Create the default rendering shader
    let ss = ShaderSettings::default()
        .source("defaults\\shaders\\rendering\\passthrough.vrsh.glsl")
        .source("defaults\\shaders\\rendering\\passthrough.vrsh.glsl");
    let shader = pipec::construct(Shader::new(ss).unwrap(), pipeline);

    // Create the default material
    let material = pipec::construct(Material::default().set_shader(shader), pipeline);

    // Create the default model
    let model = pipec::construct(Model::default(), pipeline);

    DefaultPipelineObjects {
        diffuse_tex: missing,
        normals_tex: normals,
        shader,
        material,
        model,
    }
}
// Initialize GLFW and the Window
fn init_glfw(glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
    // Set the type of events that we want to listen to
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_scroll_polling(true);
    window.set_size_polling(true);
    glfw.set_swap_interval(glfw::SwapInterval::None);
    window.make_current();
}
// Initialize OpenGL
unsafe fn init_opengl() {
    gl::Viewport(0, 0, 1280, 720);
    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    gl::Enable(gl::DEPTH_TEST);
    gl::Enable(gl::CULL_FACE);
    gl::CullFace(gl::BACK);
}
// Create the new render thread, and return some data so we can access it from other threads
pub fn init_pipeline(glfw: &mut glfw::Glfw, window: &mut glfw::Window) -> PipelineStartData {
    println!("Initializing RenderPipeline...");
    // Create a single channel to allow us to receive Pipeline Tasks from the other threads
    let (tx, rx) = std::sync::mpsc::channel::<(PipelineTask, TaskID)>(); // Main to render

    // Barrier so we can sync with the main thread at the start of each frame
    let sbarrier = Arc::new(Barrier::new(2));
    let sbarrier_clone = sbarrier.clone();

    // Barrier so we can sync with the main thread at the end of each frame
    let ebarrier = Arc::new(Barrier::new(2));
    let ebarrier_clone = ebarrier.clone();

    // An init barrier
    let ibarrier = Arc::new(Barrier::new(2));
    let ibarrier_clone = ibarrier.clone();

    // Create a simple unsafe wrapper so we can send the glfw and window data to the render thread
    // Window and GLFW wrapper
    struct RenderWrapper(AtomicPtr<glfw::Glfw>, AtomicPtr<glfw::Window>);
    let wrapper = unsafe {
        // Create the render wrapper
        let glfw = glfw as *mut glfw::Glfw;
        let window = window as *mut glfw::Window;
        unsafe impl Send for RenderWrapper {}
        unsafe impl Sync for RenderWrapper {}
        RenderWrapper(AtomicPtr::new(glfw), AtomicPtr::new(window))
    };

    // Actually make the render thread
    let handle = std::thread::spawn(move || {
        // Start OpenGL
        let glfw = unsafe { &mut *wrapper.0.load(std::sync::atomic::Ordering::Relaxed) };
        let window = unsafe { &mut *wrapper.1.load(std::sync::atomic::Ordering::Relaxed) };
        // Initialize OpenGL
        println!("Initializing OpenGL...");
        window.make_current();
        unsafe {
            glfw::ffi::glfwMakeContextCurrent(window.window_ptr());
            gl::load_with(|s| window.get_proc_address(s) as *const _);
        }

        // Init Glfw and OpenGL
        init_glfw(glfw, window);
        if gl::Viewport::is_loaded() {
            unsafe {
                init_opengl();
            }
            println!("Successfully initialized OpenGL!");
        } else {
            /* NON */
            panic!()
        }

        // Create the pipeline
        let pipeline = Pipeline::default();

        // Create the Arc and RwLock for the pipeline
        let pipeline = Arc::new(RwLock::new(pipeline));

        // Load the default objects
        {
            let mut pipeline = pipeline.write().unwrap();
            pipeline.defaults = Some(load_defaults(&*pipeline));
        }

        // Setup the pipeline renderer
        let renderer = {
            let mut pipeline = pipeline.write().unwrap();
            let mut renderer = PipelineRenderer::default();
            renderer.initialize(&mut *pipeline);
            renderer
        };

        // Set the global sender
        sender::set_global_sender(tx);

        // ---- Finished initializing the Pipeline! ----
        ibarrier_clone.wait();
        println!("Successfully created the RenderThread!");

        // We must render every frame
        loop {
            // This is a single frame
            {
                // At the start of each frame we must sync up with the main thread
                sbarrier_clone.wait();

                // We render the world here
                let pipeline = pipeline.read().unwrap();

                // And we also sync at the end of each frame
                ebarrier_clone.wait();
            }
            // This is the "free-zone". A time between the end barrier sync and the start barrier sync where we can do whatever we want with the pipeline
            {
                let mut pipeline = pipeline.write().unwrap(); // We poll the messages, buffer them, and execute them
                let messages = rx.try_iter().collect::<Vec<(PipelineTask, TaskID)>>();
                // Set the buffer
                pipeline.add_tasks(messages);

                // Execute the tasks
                pipeline.flush();

                // Check if we must exit from the render thread
                if pipeline.should_quit {
                    break;
                }
            }
        }
        println!("Stopped the render thread!");
    });
    // Wait for the init message...
    let i = std::time::Instant::now();
    println!("Waiting for RenderThread init confirmation...");
    ibarrier.wait();
    println!("Successfully initialized the RenderPipeline! Took {}ms to init RenderThread", i.elapsed().as_millis());

    // Create the pipeline start data
    PipelineStartData { handle, sbarrier, ebarrier }
}
