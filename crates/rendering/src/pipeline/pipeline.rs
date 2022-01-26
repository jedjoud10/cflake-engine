use crate::{
    advanced::compute::{ComputeShader, ComputeShaderExecutionSettings},
    basics::{
        material::Material,
        model::{Model, ModelBuffers},
        renderer::Renderer,
        shader::{Shader, ShaderSettings, ShaderSource, ShaderSourceType},
        texture::{calculate_size_bytes, get_ifd, Texture, TextureAccessType, TextureFilter, TextureReadBytes, TextureType, TextureWrapping},
        uniforms::{ShaderUniformsGroup, ShaderUniformsSettings},
    },
    object::{GlTracker, ObjectBuildingTask, ObjectID, PipelineTask, PipelineTaskCombination, PipelineTrackedTask, TrackedTaskID},
    pipeline::{camera::Camera, pipec, sender, PipelineRenderer},
    utils::{RenderWrapper, Window},
};
use ahash::{AHashMap, AHashSet};
use glfw::Context;
use ordered_vec::shareable::ShareableOrderedVec;
use std::{
    collections::HashMap,
    ffi::{c_void, CString},
    mem::size_of,
    ptr::{null, null_mut},
    sync::{
        atomic::{AtomicBool, AtomicPtr, Ordering},
        Arc, Barrier, Mutex, RwLock,
    },
};

// Some default values like the default material or even the default shader
pub(crate) struct DefaultPipelineObjects {
    pub(crate) missing_tex: ObjectID<Texture>,
    pub(crate) black: ObjectID<Texture>,
    pub(crate) white: ObjectID<Texture>,
    pub(crate) normals_tex: ObjectID<Texture>,
    pub(crate) shader: ObjectID<Shader>,
    pub(crate) material: ObjectID<Material>,
    pub(crate) model: ObjectID<Model>,
}

// Some internal pipeline data that we store on the render thread and that we cannot share with the other threads
#[derive(Default)]
pub(crate) struct InternalPipeline {
    // Keep track of the GlTrackers and their corresponding ID
    gltrackers: AHashMap<TrackedTaskID, GlTracker>,
}

// The rendering pipeline. It can be shared around using Arc, but we are only allowed to modify it on the Render Thread
// This is only updated at the end of each frame, so we don't have to worry about reading it from multiple threads since no one will be writing to it at that times
#[derive(Default)]
pub struct Pipeline {
    // We will buffer the tasks, so that way whenever we receive a task internally from the Render Thread itself we can just wait until we manually flush the tasks to execute all at once
    tasks: RwLock<Vec<PipelineTaskCombination>>,

    // Tracked tasks
    completed_tracked_tasks: AHashSet<TrackedTaskID>,
    pub(crate) completed_finalizers: AHashSet<TrackedTaskID>,

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

    // The current main camera that is rendering the world
    pub(crate) camera: Camera,
    // Our window
    pub window: Window,
    // Atomic used to debug some data
    pub(crate) debugging: AtomicBool,
}

impl Pipeline {
    // Set the buffered tasks from RX messages
    pub(crate) fn add_tasks(&mut self, messages: Vec<PipelineTaskCombination>) {
        let mut write = self.tasks.write().unwrap();
        for task in messages {
            write.push(task);
        }
    }
    // Add a task interally, through the render thread itself
    pub(crate) fn add_task_internally(&self, task: PipelineTaskCombination) {
        let mut write = self.tasks.write().unwrap();
        write.push(task);
    }
    // Execute a single task
    fn execute_task(&mut self, renderer: &mut PipelineRenderer, task: PipelineTask) {
        // Now we must execute these tasks
        match task {
            // Creation tasks
            PipelineTask::CreateTexture(id) => self.texture_create(id),
            PipelineTask::CreateMaterial(id) => self.material_create(id),
            PipelineTask::CreateShader(id) => self.shader_create(id),
            PipelineTask::CreateModel(id) => self.model_create(id),
            PipelineTask::CreateRenderer(id) => self.renderer_create(id),
            PipelineTask::CreateComputeShader(id) => self.compute_create(id),

            PipelineTask::UpdateRendererMatrix(id, matrix) => self.renderer_update_matrix(id, matrix),
            PipelineTask::UpdateCamera(camera) => self.camera = camera,
            PipelineTask::UpdateTextureDimensions(id, tt) => self.texture_update_size(id, tt),

            // Window tasks
            PipelineTask::SetWindowDimension(new_dimensions) => self.set_window_dimension(renderer, new_dimensions),
            PipelineTask::SetWindowFocusState(focused) => self.set_window_focus_state(focused),
        }
    }
    // Check if the awaiting tracked task of a tracked task have fully completed
    fn awaits_completed(&self, awaits: &Vec<TrackedTaskID>) -> bool {
        awaits.iter().all(|id| self.completed_tracked_tasks.contains(id)) || awaits.is_empty()
    }
    // Execute a single tracked task, but also create a respective OpenGL fence for said task
    fn execute_tracked_task(&mut self, internal: &mut InternalPipeline, renderer: &mut PipelineRenderer, task: PipelineTrackedTask, tracking_id: TrackedTaskID) {
        // Create a corresponding GlTracker for this task
        let gltracker = match task {
            PipelineTrackedTask::RunComputeShader(id, settings) => self.compute_run(id, settings),
            PipelineTrackedTask::TextureReadBytes(id, read) => self.fill_texture(id, read),
            PipelineTrackedTask::TextureWriteBytes(id, write) => todo!(),
        };

        // Add the tracked ID to our pipeline
        internal.gltrackers.insert(tracking_id, gltracker);
    }
    // Finalizer for the tracked task
    fn handle_tracked_finalizer(&mut self, tracking_id: TrackedTaskID, requirements: Vec<TrackedTaskID>) {
        // Add the finalizer since all the required tasks have finished executing
        self.completed_finalizers.insert(tracking_id);
        // And clean the old tasks
        for require in requirements {
            self.completed_tracked_tasks.remove(&require);
        }
    }
    // Called each frame during the "free-zone"
    pub(crate) fn update(&mut self, internal: &mut InternalPipeline, renderer: &mut PipelineRenderer) {
        // Also check each GlTracker and check if it finished executing
        let completed = internal.gltrackers.drain_filter(|id, tracker| tracker.completed(self)).collect::<Vec<_>>();
        for (id, _) in completed {
            self.completed_tracked_tasks.insert(id);
        }
        // Clean the completed tracked finalizers
        self.completed_finalizers.clear();
        // Always flush during the update
        self.flush(internal, renderer);
    }
    // Flush all the buffered tasks, and execute them
    // We should do this at the end of each frame, but we can force execution of it if we are running it internally
    pub(crate) fn flush(&mut self, internal: &mut InternalPipeline, renderer: &mut PipelineRenderer) {
        // We must take the commands first
        let tasks = {
            let mut tasks_ = self.tasks.write().unwrap();
            let tasks = &mut *tasks_;
            // If we have a tracked task that requires the execution of another tracked task, we must check if the latter has completed executing
            let tasks = tasks
                .drain_filter(|task| match task {
                    PipelineTaskCombination::SingleTracked(_, _, require) => {
                        // If the requirement is null, that means that we don't need it
                        let valid = require.and_then(|x| if self.completed_tracked_tasks.contains(&x) { None } else { Some(()) });
                        valid.is_none()
                    }
                    PipelineTaskCombination::SingleTrackedFinalizer(_, requirements) => {
                        // Check each task
                        if requirements.is_empty() {
                            panic!();
                        }
                        let valid = requirements.into_iter().all(|x| self.completed_tracked_tasks.contains(&x));
                        valid
                    }
                    _ => true,
                })
                .collect::<Vec<_>>();
            tasks
        };

        for task in tasks {
            match task {
                PipelineTaskCombination::Single(task) => self.execute_task(renderer, task),
                PipelineTaskCombination::Batch(batch) => {
                    // Execute all the tasks
                    for task in batch {
                        self.execute_task(renderer, task);
                    }
                }

                PipelineTaskCombination::SingleTracked(task, tracking_id, _) => self.execute_tracked_task(internal, renderer, task, tracking_id),
                PipelineTaskCombination::SingleTrackedFinalizer(tracking_id, requirements) => self.handle_tracked_finalizer(tracking_id, requirements),
            }
        }

        // Update the window if needed
        let update_window = self.window.update.load(Ordering::Relaxed);
        if update_window {
            let (glfw, window) = (self.window.wrapper.0.load(Ordering::Relaxed), self.window.wrapper.1.load(Ordering::Relaxed));
            let (glfw, _window) = unsafe { (&mut *glfw, &mut *window) };
            if self.window.vsync.load(Ordering::Relaxed) {
                glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
            } else {
                glfw.set_swap_interval(glfw::SwapInterval::None);
            }
        }
    }
    // Set the global shader uniforms
    pub(crate) fn update_global_shader_uniforms(&mut self, time: f64, delta: f64) {
        for (id, _shader) in self.shaders.iter() {
            // Set the uniforms
            let mut group = ShaderUniformsGroup::new();
            group.set_f64("_time", time);
            group.set_f64("_delta", delta);
            let id = ShaderUniformsSettings::new(ObjectID::new(id));
            group.execute(self, id).unwrap();
        }
    }
    // Get a material using it's respective ID
    pub fn get_material(&self, id: ObjectID<Material>) -> Option<&Material> {
        if let Some(id) = id.id {
            self.materials.get(id)
        } else {
            None
        }
    }
    // Get a model using it's respective ID
    pub fn get_model(&self, id: ObjectID<Model>) -> Option<&(Model, ModelBuffers)> {
        if let Some(id) = id.id {
            self.models.get(id)
        } else {
            None
        }
    }
    // Get a renderer using it's respective ID
    pub fn get_renderer(&self, id: ObjectID<Renderer>) -> Option<&Renderer> {
        if let Some(id) = id.id {
            self.renderers.get(id)
        } else {
            None
        }
    }
    // Get a shader using it's respective ID
    pub fn get_shader(&self, id: ObjectID<Shader>) -> Option<&Shader> {
        if let Some(id) = id.id {
            self.shaders.get(id)
        } else {
            None
        }
    }
    // Get a compute shader using it's respective ID
    pub fn get_compute_shader(&self, id: ObjectID<ComputeShader>) -> Option<&ComputeShader> {
        if let Some(id) = id.id {
            self.compute_shaders.get(id)
        } else {
            None
        }
    }
    // Get a texture using it's texture
    pub fn get_texture(&self, id: ObjectID<Texture>) -> Option<&Texture> {
        if let Some(id) = id.id {
            self.textures.get(id)
        } else {
            None
        }
    }
    // Mutable
    // Get a mutable material using it's respective ID
    pub(crate) fn get_material_mut(&mut self, id: ObjectID<Material>) -> Option<&mut Material> {
        if let Some(id) = id.id {
            self.materials.get_mut(id)
        } else {
            None
        }
    }
    // Get a mutable model using it's respective ID
    pub(crate) fn get_model_mut(&mut self, id: ObjectID<Model>) -> Option<&mut (Model, ModelBuffers)> {
        if let Some(id) = id.id {
            self.models.get_mut(id)
        } else {
            None
        }
    }
    // Get a renderer using it's respective ID
    pub(crate) fn get_renderer_mut(&mut self, id: ObjectID<Renderer>) -> Option<&mut Renderer> {
        if let Some(id) = id.id {
            self.renderers.get_mut(id)
        } else {
            None
        }
    }
    // Get a shader using it's respective ID
    pub(crate) fn get_shader_mut(&mut self, id: ObjectID<Shader>) -> Option<&mut Shader> {
        if let Some(id) = id.id {
            self.shaders.get_mut(id)
        } else {
            None
        }
    }
    // Get a compute shader using it's respective ID
    pub(crate) fn get_compute_shader_mut(&mut self, id: ObjectID<ComputeShader>) -> Option<&mut ComputeShader> {
        if let Some(id) = id.id {
            self.compute_shaders.get_mut(id)
        } else {
            None
        }
    }
    // Get a texture using it's texture
    pub(crate) fn get_texture_mut(&mut self, id: ObjectID<Texture>) -> Option<&mut Texture> {
        if let Some(id) = id.id {
            self.textures.get_mut(id)
        } else {
            None
        }
    }

    // Actually update our data
    // Add the renderer
    fn renderer_create(&mut self, task: ObjectBuildingTask<Renderer>) {
        // Get the renderer data, if it does not exist then use the default renderer data
        let renderer = task.0;
        let defaults = self.defaults.as_ref().unwrap();
        let _material_id = self.get_material(defaults.material);
        let _model_id = self.get_model(defaults.model);

        self.renderers.insert(task.1.id.unwrap(), renderer);
    }
    // Remove the renderer using it's renderer ID
    fn renderer_dispose(&mut self, id: ObjectID<Renderer>) {
        self.renderers.remove(id.id.unwrap());
    }
    // Update a renderer's matrix
    fn renderer_update_matrix(&mut self, id: ObjectID<Renderer>, matrix: veclib::Matrix4x4<f32>) {
        let renderer = self.renderers.get_mut(id.id.unwrap()).unwrap();
        renderer.matrix = matrix;
    }
    // Create a shader and cache it. We do not cache the "subshader" though
    fn shader_create(&mut self, task: ObjectBuildingTask<Shader>) {
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
        let program = unsafe {
            let program = gl::CreateProgram();

            // Create & compile the shader sources and link them
            let taken = std::mem::take(&mut shader.sources);
            let programs: Vec<u32> = taken.into_iter().map(|(_path, data)| compile_single_source(data)).collect::<Vec<_>>();
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
            program
        };
        // Add the shader at the end
        shader.program = program;
        self.shaders.insert(task.1.id.unwrap(), shader);
    }
    // Create a compute shader and cache it
    fn compute_create(&mut self, task: ObjectBuildingTask<ComputeShader>) {
        // Extract the shader
        let mut shader = task.0;

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
        let program = unsafe {
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
            program
        };
        // Add the shader at the end
        shader.program = program;
        self.compute_shaders.insert(task.1.id.unwrap(), shader);
    }
    // Create a model
    fn model_create(&mut self, task: ObjectBuildingTask<Model>) {
        let model = task.0;
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
        self.models.insert(task.1.id.unwrap(), (model, buffers));
    }
    // Dispose of a model, also remove it from the pipeline
    fn model_dispose(&mut self, id: ObjectID<Model>) {
        // Remove the model and it's buffers
        let (_model, mut buffers) = self.models.remove(id.id.unwrap()).unwrap();
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
    fn texture_create(&mut self, task: ObjectBuildingTask<Texture>) {
        // Guess how many mipmap levels a texture with a specific maximum coordinate can have
        fn guess_mipmap_levels(i: usize) -> usize {
            let mut x: f32 = i as f32;
            let mut num: usize = 0;
            while x > 1.0 {
                // Repeatedly divide by 2
                x /= 2.0;
                num += 1;
            }
            num
        }

        let mut texture = task.0;
        let mut pointer: *const c_void = null();
        if !texture.bytes.is_empty() {
            pointer = texture.bytes.as_ptr() as *const c_void;
        }
        let ifd = get_ifd(texture._format, texture._type);
        let bytes_count = calculate_size_bytes(&texture._format, texture.count_pixels());

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
                        guess_mipmap_levels(width.max(height) as usize) as i32,
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
        if texture.mipmaps {
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

        // Create the Upload / Download PBOs if needed
        if texture.cpu_access.contains(TextureAccessType::READ) {
            // Create a download PBO
            let mut pbo = 0_u32;
            unsafe {
                gl::GenBuffers(1, &mut pbo);
                gl::BindBuffer(gl::PIXEL_PACK_BUFFER, pbo);
                gl::BufferData(gl::PIXEL_PACK_BUFFER, bytes_count as isize, null(), gl::STREAM_COPY);
                gl::BindBuffer(gl::PIXEL_PACK_BUFFER, 0);
            }
            texture.read_pbo = Some(pbo);
        } else if texture.cpu_access.contains(TextureAccessType::WRITE) {
            // Create an upload PBO
            let mut pbo = 0_u32;
            unsafe {
                gl::GenBuffers(1, &mut pbo);
                gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, pbo);
                gl::BufferData(gl::PIXEL_UNPACK_BUFFER, bytes_count as isize, null(), gl::STREAM_DRAW);
                gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, 0);
            }
            texture.write_pbo = Some(pbo);
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
        texture.oid = id;
        self.textures.insert(task.1.id.unwrap(), texture);
    }
    // Update the size of a texture
    // PS: This also clears the texture
    fn texture_update_size(&mut self, id: ObjectID<Texture>, tt: TextureType) {
        // Get the GPU texture object
        let texture = self.get_texture_mut(id).unwrap();
        // Check if the current dimension type matches up with the new one
        texture.ttype = tt;
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
    // Run a compute shader, and return it's GlTracker
    fn compute_run(&mut self, id: ObjectID<ComputeShader>, settings: ComputeShaderExecutionSettings) -> GlTracker {
        // Execute some shader uniforms if we want to
        let group = settings.uniforms;
        if let Some(group) = group {
            // Create some shader uniforms settings that we can use
            let settings = ShaderUniformsSettings::new_compute(id);
            group.execute(self, settings).unwrap();
        }
        // Dispatch the compute shader for execution
        let axii = settings.axii;

        // Create the GlTracker and send the DispatchCompute command
        GlTracker::new(
            |_| unsafe {
                gl::DispatchCompute(axii.0 as u32, axii.1 as u32, axii.2 as u32);
            },
            |_| {},
            self,
        )
    }
    // Read the bytes from a texture
    fn fill_texture(&mut self, id: ObjectID<Texture>, read: TextureReadBytes) -> GlTracker {
        // Actually read the pixels
        GlTracker::new(
            |pipeline| unsafe {
                // Bind the buffer before reading
                let texture = pipeline.get_texture(id).unwrap();
                gl::BindBuffer(gl::PIXEL_PACK_BUFFER, texture.read_pbo.unwrap());
                gl::BindTexture(texture.target, texture.oid);
                let (_internal_format, format, data_type) = texture.ifd;
                gl::GetTexImage(texture.target, 0, format, data_type, null_mut());
            },
            move |pipeline| unsafe {
                // Gotta create a mapped buffer
                let texture = pipeline.get_texture(id).unwrap();
                let byte_count = calculate_size_bytes(&texture._format, texture.count_pixels());
                let mut vec = vec![0_u8; byte_count];
                gl::BindBuffer(gl::PIXEL_PACK_BUFFER, texture.read_pbo.unwrap());
                gl::GetBufferSubData(gl::PIXEL_PACK_BUFFER, 0, byte_count as isize, vec.as_mut_ptr() as *mut c_void);
                let mut cpu_bytes = read.cpu_bytes.as_ref().lock().unwrap();
                *cpu_bytes = vec;
            },
            self,
        )
    }
    // Create a materail
    fn material_create(&mut self, task: ObjectBuildingTask<Material>) {
        // Just add the material internally
        self.materials.insert(task.1.id.unwrap(), task.0);
    }
    // Update the window dimensions
    fn set_window_dimension(&mut self, renderer: &mut PipelineRenderer, new_dimensions: veclib::Vector2<u16>) {
        self.window.dimensions = new_dimensions;
        renderer.update_window_dimensions(new_dimensions, self);
    }
    // Set the focus state for our window
    fn set_window_focus_state(&mut self, focused: bool) {
        self.window.focused = focused;
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
    // An atomic we use to shutdown the render thread
    pub eatomic: Arc<AtomicBool>,
    // The pipeline itself
    pub pipeline: Arc<RwLock<Pipeline>>,
    // Some timing data that we will share with the pipeline
    pub time: Arc<Mutex<(f64, f64)>>,
}
// Load some defaults
fn load_defaults(pipeline: &Pipeline) -> DefaultPipelineObjects {
    use assets::assetc::load;

    // Create the default missing texture
    let missing = pipec::construct(load("defaults\\textures\\missing_texture.png", Texture::default().set_mipmaps(true)).unwrap(), pipeline);

    // Create the default white texture
    let white = pipec::construct(
        Texture::default()
            .set_dimensions(TextureType::Texture2D(1, 1))
            .set_filter(TextureFilter::Linear)
            .set_bytes(vec![255, 255, 255, 255])
            .set_mipmaps(true),
        pipeline,
    );

    // Create the default black texture
    let black = pipec::construct(
        Texture::default()
            .set_dimensions(TextureType::Texture2D(1, 1))
            .set_filter(TextureFilter::Linear)
            .set_bytes(vec![0, 0, 0, 255])
            .set_mipmaps(true),
        pipeline,
    );

    // Create the default normal map texture
    let normals = pipec::construct(
        Texture::default()
            .set_dimensions(TextureType::Texture2D(1, 1))
            .set_filter(TextureFilter::Linear)
            .set_bytes(vec![127, 128, 255, 255])
            .set_mipmaps(true),
        pipeline,
    );

    // Create the default rendering shader
    let ss = ShaderSettings::default()
        .source("defaults\\shaders\\rendering\\default.vrsh.glsl")
        .source("defaults\\shaders\\rendering\\default.frsh.glsl");
    let shader = pipec::construct(Shader::new(ss).unwrap(), pipeline);

    // Create the default material
    let mut material = Material::default().set_shader(shader);
    material.set_pre_construct_settings(missing, black, normals);
    let material = pipec::construct_only(material, pipeline);

    // Create the default model
    let model = pipec::construct(Model::default(), pipeline);

    DefaultPipelineObjects {
        missing_tex: missing,
        black,
        white,
        normals_tex: normals,
        shader,
        material,
        model,
    }
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
    let (tx, rx) = std::sync::mpsc::channel::<PipelineTaskCombination>(); // Main to render

    // Barrier so we can sync with the main thread at the start of each frame
    let sbarrier = Arc::new(Barrier::new(2));
    let sbarrier_clone = sbarrier.clone();

    // Barrier so we can sync with the main thread at the end of each frame
    let ebarrier = Arc::new(Barrier::new(2));
    let ebarrier_clone = ebarrier.clone();

    // An atomic that we use to inform the render thread to exit and shutdown
    let eatomic = Arc::new(AtomicBool::new(false));
    let eatomic_clone = eatomic.clone();

    // Some timing data that we will share with the pipeline
    let time = Arc::new(Mutex::new((0.0, 0.0)));
    let time_clone = time.clone();

    // An init channel
    let (itx, irx) = std::sync::mpsc::sync_channel::<Arc<RwLock<Pipeline>>>(1);

    // The main thread will not own the glfw context, and we will send it to the render thread instead
    unsafe { glfw::ffi::glfwMakeContextCurrent(null_mut()) }

    // Window and GLFW wrapper
    let wrapper = {
        // Create the render wrapper
        let glfw = glfw as *mut glfw::Glfw;
        let window = window as *mut glfw::Window;
        Arc::new(RenderWrapper(AtomicPtr::new(glfw), AtomicPtr::new(window)))
    };

    // Actually make the render thread
    let handle = std::thread::spawn(move || {
        // Start OpenGL
        let _glfw = unsafe { &mut *wrapper.0.load(std::sync::atomic::Ordering::Relaxed) };
        let window = unsafe { &mut *wrapper.1.load(std::sync::atomic::Ordering::Relaxed) };
        // Initialize OpenGL
        println!("Initializing OpenGL...");
        window.make_current();
        unsafe {
            glfw::ffi::glfwMakeContextCurrent(window.window_ptr());
            gl::load_with(|s| window.get_proc_address(s) as *const _);
        }
        if gl::Viewport::is_loaded() {
            unsafe {
                init_opengl();
            }
            println!("Successfully initialized OpenGL!");
        } else {
            /* NON */
            panic!()
        }
        // Window wrapper
        let window_wrapper = Window::new(wrapper.clone());

        // Set the global sender
        sender::set_global_sender(tx);

        // Create the pipeline
        let pipeline = Pipeline::default();
        // Create an internal pipeline as well
        let mut internal = InternalPipeline::default();

        // Create the Arc and RwLock for the pipeline
        let pipeline = Arc::new(RwLock::new(pipeline));

        // Load the default objects
        {
            let mut pipeline = pipeline.write().unwrap();
            pipeline.window = window_wrapper;
            pipeline.defaults = Some(load_defaults(&*pipeline));
        }

        // Setup the pipeline renderer
        let mut renderer = {
            let mut pipeline = pipeline.write().unwrap();
            let mut renderer = PipelineRenderer::default();
            renderer.initialize(&mut internal, &mut *pipeline);
            renderer
        };

        // ---- Finished initializing the Pipeline! ----
        itx.send(pipeline.clone()).unwrap();
        println!("Successfully created the RenderThread!");

        // We must render every frame
        loop {
            let i = std::time::Instant::now();
            {
                // At the start of each frame we must sync up with the main thread
                sbarrier_clone.wait();
            }
            // This is a single frame
            {
                let mut pipeline = pipeline.write().unwrap();
                let time = time_clone.lock().unwrap();
                pipeline.update_global_shader_uniforms(time.0, time.1);
            }
            {
                // We render the world here
                let pipeline = pipeline.read().unwrap();
                renderer.pre_render();
                renderer.render_frame(&*pipeline);
                renderer.post_render(&*pipeline);
                // Do not forget to switch buffers at the end of the frame
                window.swap_buffers();

                // And we also sync at the end of each frame
                ebarrier_clone.wait();
            }
            // This is the "free-zone". A time between the end barrier sync and the start barrier sync where we can do whatever we want with the pipeline
            {
                let mut pipeline = pipeline.write().unwrap(); // We poll the messages, buffer them, and execute them
                let messages = rx.try_iter().collect::<Vec<PipelineTaskCombination>>();
                // Set the buffer
                pipeline.add_tasks(messages);

                // Execute the tasks
                pipeline.update(&mut internal, &mut renderer);

                // Debug if needed
                if pipeline.debugging.load(Ordering::Relaxed) {
                    println!("Pipeline Frame Time: {:.2}ms", i.elapsed().as_secs_f32() * 1000.0);
                }

                // Check if we must exit from the render thread
                if eatomic_clone.load(Ordering::Relaxed) {
                    break;
                }
            }
        }
        println!("Stopped the render thread!");
    });
    // Wait for the init message...
    let i = std::time::Instant::now();
    println!("Waiting for RenderThread init confirmation...");
    let pipeline = irx.recv().unwrap();
    println!("Successfully initialized the RenderPipeline! Took {}ms to init RenderThread", i.elapsed().as_millis());
    /*
    glfw.with_primary_monitor_mut(|glfw, monitor| {
        let videomode = monitor.unwrap().get_video_mode().unwrap();
        window.set_monitor(WindowMode::Windowed, 0, 0, videomode.width, videomode.height, Some(videomode.refresh_rate));
    });
    */
    // Create the pipeline start data
    PipelineStartData {
        handle,
        sbarrier,
        ebarrier,
        eatomic,
        pipeline,
        time,
    }
}
