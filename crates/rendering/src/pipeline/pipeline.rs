
use glfw::Context;

use super::object::*;
use crate::{basics::*, pipec, rendering::PipelineRenderer, RenderCommand, RenderTask, RenderTaskReturn, RenderTaskStatus, SharedData, SpecialPipelineMessage};
use std::{
    collections::HashMap,
    ffi::{c_void, CString},
    mem::size_of,
    ptr::null,
    sync::{
        mpsc::{Receiver, Sender},
    },
};

// Run a command on the Render Thread
fn command(command: RenderCommand) -> RenderTaskReturn {
    // Handle the common cases
    match command.input_task {
        // Window
        RenderTask::WindowSizeUpdate(_width, _height, _aspect_ratio) => todo!(),
        // Pipeline
        RenderTask::DestroyRenderThread() => todo!(),
        RenderTask::CameraDataUpdate(_) => RenderTaskReturn::None,
        // Shaders
        RenderTask::SubShaderCreate(shared_shader) => RenderTaskReturn::GPUObject(Pipeline::create_compile_subshader(shared_shader)),
        RenderTask::ShaderCreate(shared_shader) => RenderTaskReturn::GPUObject(Pipeline::create_compile_shader(shared_shader)),
        RenderTask::ShaderUniformGroup(_shared_uniformgroup) => todo!(),
        // Textures
        RenderTask::TextureCreate(shared_texture) => RenderTaskReturn::GPUObject(Pipeline::generate_texture(shared_texture)),
        RenderTask::TextureUpdateSize(texture, ttype) => {
            Pipeline::update_texture_size(texture, ttype);
            RenderTaskReturn::None
        }
        RenderTask::TextureUpdateData(texture, bytes) => {
            Pipeline::update_texture_data(texture, bytes);
            RenderTaskReturn::None
        }
        RenderTask::TextureFillArray(texture, bytecount) => RenderTaskReturn::TextureFillData(Pipeline::texture_fill_array(texture, bytecount)),
        // Model
        RenderTask::ModelCreate(shared_model) => RenderTaskReturn::GPUObject(Pipeline::create_model(shared_model)),
        RenderTask::ModelDispose(gpumodel) => {
            Pipeline::dispose_model(gpumodel);
            RenderTaskReturn::None
        }
        // Compute
        RenderTask::ComputeRun(compute, indices) => {
            Pipeline::run_compute(compute, indices);
            RenderTaskReturn::None
        }
        RenderTask::ComputeLock(compute) => {
            Pipeline::lock_compute(compute);
            RenderTaskReturn::None
        }
        // Renderer
        RenderTask::RendererAdd(_) => RenderTaskReturn::None,
        RenderTask::RendererRemove(_) => RenderTaskReturn::None,
        RenderTask::RendererUpdateTransform(_renderer, _transform) => todo!(),
    }
}

// Poll commands that have been sent to us by the main thread
fn poll_commands(
    pr: &mut PipelineRenderer,
    camera: &mut CameraDataGPUObject,
    main_to_render: &Receiver<RenderCommand>,
    valid: &mut bool,
    render_to_main: &Sender<RenderTaskStatus>,
) {
    // We must loop through every command that we receive from the main thread
    for _command in main_to_render.try_iter() {
        // Check special commands first
        let message_id = _command.message_id;
        match _command.input_task {
            RenderTask::DestroyRenderThread() => {
                // Destroy the render thread
                *valid = false;
                println!("Destroy RenderThread and RenderPipeline!");
                break;
            }
            // Renderer commands
            RenderTask::RendererAdd(shared_renderer) => {
                println!("Add renderer");
                let gpuobject = RenderTaskReturn::GPUObject(Pipeline::add_renderer(pr, shared_renderer));
                render_to_main.send(RenderTaskStatus::Successful(gpuobject, message_id)).unwrap();
            }
            RenderTask::RendererRemove(renderer_id) => Pipeline::remove_renderer(pr, renderer_id),
            // Camera update command
            RenderTask::CameraDataUpdate(shared) => {
                let pos = shared.object.0;
                let rot = shared.object.1;
                let clip_planes = shared.object.2;
                let projm = shared.object.3;
                // Calculate the view matrix using the position and rotation
                let viewm = {
                    let rm = veclib::Matrix4x4::from_quaternion(&rot);
                    let forward_vector = rm.mul_point(&veclib::Vector3::<f32>::new(0.0, 0.0, -1.0)).normalized();
                    let up_vector = rm.mul_point(&veclib::Vector3::<f32>::new(0.0, 1.0, 0.0)).normalized();
                    veclib::Matrix4x4::look_at(&pos, &up_vector, &(forward_vector + pos))
                };
                *camera = CameraDataGPUObject {
                    position: pos,
                    rotation: rot,
                    clip_planes,
                    viewm,
                    projm,
                }
            }
            _ => {
                // Valid command
                match command(_command) {
                    RenderTaskReturn::None | RenderTaskReturn::NoneUnwaitable => { /* Fat bruh */ }
                    x => {
                        // Valid
                        render_to_main.send(RenderTaskStatus::Successful(x, message_id)).unwrap();
                    }
                }
            }
        }
    }
}

// The render thread that is continuously being ran
fn frame(
    _glfw: &mut glfw::Glfw,
    window: &mut glfw::Window,
    render_to_main: &Sender<RenderTaskStatus>,
    main_to_render: &Receiver<RenderCommand>,
    pipeline_renderer: &mut PipelineRenderer,
    camera: &mut CameraDataGPUObject,
    valid: &mut bool,
    _frame_count: u128,
    _delta_time: f64,
) {
    // Poll first
    poll_commands(pipeline_renderer, camera, main_to_render, valid, render_to_main);
    // Pre-render
    pipeline_renderer.pre_render();
    // Render
    pipeline_renderer.renderer_frame(camera);
    // Post-render
    pipeline_renderer.post_render(window);
}

// Render pipeline. Contains everything related to rendering. This is also ran on a separate thread
#[derive(Default)]
pub struct Pipeline {
    pub command_id: u128,                                                        // Next Command ID
    pub render_commands_buffer: HashMap<u128, Box<dyn FnMut(RenderTaskStatus)>>, // The tasks that are asynchronous and are pending their return values
    pub gpu_objects: HashMap<u128, GPUObject>,                                   // The GPU objects that where generated on the Rendering Thread and sent back to the main thread
    pub gpu_objects_names: HashMap<String, u128>,                                // A name to GPU initialization ID
    pub render_to_main: Option<Receiver<RenderTaskStatus>>,                      // RX (MainThread)
    pub main_to_render: Option<Sender<RenderCommand>>,                           // TX (MainThread)
}
impl Pipeline {
    // Create the new render thread
    pub fn init_pipeline(&mut self, glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
        println!("Initializing RenderPipeline...");
        // Create the two channels
        let (tx, rx): (Sender<RenderTaskStatus>, Receiver<RenderTaskStatus>) = std::sync::mpsc::channel(); // Render to main
        let (tx2, rx2): (Sender<RenderCommand>, Receiver<RenderCommand>) = std::sync::mpsc::channel(); // Main to render
        let (tx3, rx3): (Sender<SpecialPipelineMessage>, Receiver<SpecialPipelineMessage>) = std::sync::mpsc::channel(); // Render to main, but for special cases
        unsafe {
            // Window and GLFW wrapper
            struct RenderWrapper(*mut glfw::Glfw, *mut glfw::Window);
            unsafe impl Send for RenderWrapper {}
            unsafe impl Sync for RenderWrapper {}
            // Create the render wrapper
            let render_wrapper = RenderWrapper(glfw as *mut glfw::Glfw, window as *mut glfw::Window);
            std::thread::spawn(move || {
                // Start OpenGL
                let glfw = &mut *render_wrapper.0;
                let window = &mut *render_wrapper.1;
                // Initialize OpenGL
                println!("Initializing OpenGL...");
                window.make_current();
                glfw::ffi::glfwMakeContextCurrent(window.window_ptr());
                gl::load_with(|s| window.get_proc_address(s) as *const _);

                // Set the type of events that we want to listen to
                window.set_key_polling(true);
                window.set_cursor_pos_polling(true);
                window.set_scroll_polling(true);
                window.set_size_polling(true);
                glfw.set_swap_interval(glfw::SwapInterval::None);
                window.make_current();
                if gl::Viewport::is_loaded() {
                    gl::Viewport(0, 0, 1280, 720);
                    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                    gl::Enable(gl::DEPTH_TEST);
                    gl::Enable(gl::CULL_FACE);
                    gl::CullFace(gl::BACK);
                    println!("Successfully initialized OpenGL!");
                } else {
                    /* NON */
                    panic!()
                }
                // Renaming a bit
                let render_to_main = tx;
                let main_to_render = rx2;

                // Initialize the deferred renderer
                let mut pipeline_renderer = PipelineRenderer::default();
                pipeline_renderer.init(veclib::Vector2::new(1280, 720));

                // El camera
                let mut camera = CameraDataGPUObject {
                    position: veclib::Vector3::ZERO,
                    rotation: veclib::Quaternion::IDENTITY,
                    clip_planes: veclib::Vector2::ZERO,
                    viewm: veclib::Matrix4x4::IDENTITY,
                    projm: veclib::Matrix4x4::IDENTITY,
                };

                // We must render every frame
                // Timing stuff
                let mut last_time: f64 = 0.0;
                let mut frame_count: u128 = 0;
                // If the render pipeline and thread are valid
                let mut valid = true;
                println!("Successfully created the RenderThread!");
                // Send a message to the main thread indicating that the RenderThread successfully launched
                tx3.send(SpecialPipelineMessage::RenderThreadInitialized).unwrap();

                while valid {
                    // Update the delta_time
                    let new_time = glfw.get_time();
                    let delta = new_time - last_time;
                    last_time = new_time;
                    // Run the frame
                    frame(
                        glfw,
                        window,
                        &render_to_main,
                        &main_to_render,
                        &mut pipeline_renderer,
                        &mut camera,
                        &mut valid,
                        frame_count,
                        delta,
                    );
                    frame_count += 1;
                }
            });
        };
        // Wait for the init message...
        let i = std::time::Instant::now();
        println!("Waiting for RenderThread init confirmation...");
        let _x = rx3.recv().unwrap();
        println!("Received RenderThread init confirmation! Took {}ms to init RenderThread", i.elapsed().as_millis());
        // Vars
        self.render_to_main = Some(rx);
        self.main_to_render = Some(tx2);
        println!("Successfully initialized the RenderPipeline!");
    }
    // Load some default rendering things
    pub fn start_world(&mut self) {
        // Default shader
        let ds = pipec::shader(
            Shader::default()
                .load_shader(vec![
                    "defaults\\shaders\\rendering\\passthrough.vrsh.glsl",
                    "defaults\\shaders\\rendering\\screen.frsh.glsl",
                ])
                .unwrap(),
        );
        // Default material
        let _dm = Material::new("Default material").set_shader(ds);
    }
    // Frame on the main thread
    pub fn frame_main_thread(&mut self) {
        // Receive the return data that was sent from the render thread
        let render_to_main = self.render_to_main.as_ref().unwrap();
        for received in render_to_main.try_iter() {
            let (task_return, message_id) = match received {
                RenderTaskStatus::Successful(x, message_id) => (x, message_id),
                RenderTaskStatus::Failed => panic!(),
            };
            // Call the callbacks
            let mut callback = self.render_commands_buffer.remove(&message_id).unwrap();
            match &task_return {
                RenderTaskReturn::GPUObject(x) => {
                    self.gpu_objects.insert(message_id, x.clone());
                }
                _ => {}
            }
            callback(RenderTaskStatus::Successful(task_return, message_id));
        }
    }
    // Dispose of the current render thread and pipeline
    pub fn dispose_pipeline(&mut self) {
        self.task_immediate(RenderTask::DestroyRenderThread());
    }
    // Complete a task immediatly
    pub fn task_immediate(&mut self, task: RenderTask) -> Option<RenderTaskReturn> {
        println!("TaskImmediate! {}", self.command_id);
        let should_wait = task.returns_to_main();
        // Create a new render command and send it to the separate thread
        let render_command = RenderCommand {
            message_id: self.command_id,
            input_task: task,
        };
        // Send the command to the render thread
        let tx = self.main_to_render.as_ref().unwrap();
        let rx = self.render_to_main.as_ref().unwrap();

        // Send the command
        tx.send(render_command).unwrap();
        // Increment
        self.command_id += 1;
        // Wait for the result (only if we need to)
        if should_wait {
            let output = match rx.recv().ok()? {
                RenderTaskStatus::Successful(x, _) => Some(x),
                RenderTaskStatus::Failed => None,
            };
            println!("TaskImmediate {} success!", self.command_id - 1);
            output
        } else {
            println!("TaskImmediate {} success! Though did not need to wait", self.command_id - 1);
            Some(RenderTaskReturn::NoneUnwaitable)
        }
    }
    // Complete a task, but the result is not needed immediatly, and call the call back when the task finishes
    pub fn task<F>(&mut self, task: RenderTask, callback: F)
    where
        F: FnMut(RenderTaskStatus) + 'static,
    {
        println!("Task! {}", self.command_id);
        let boxed_fn_mut: Box<dyn FnMut(RenderTaskStatus)> = Box::new(callback);
        // Create a new render command and send it to the separate thread
        let render_command = RenderCommand {
            message_id: self.command_id,
            input_task: task,
        };
        // Send the command to the render thread
        let tx = self.main_to_render.as_ref().unwrap();

        // Send the command
        tx.send(render_command).unwrap();
        // This time, we must add this to the wait list
        self.render_commands_buffer.insert(self.command_id, boxed_fn_mut);
        // Increment
        self.command_id += 1;
        println!("Task! {} succsess", self.command_id);
    }
    // Internal immediate task. This assumes that we are already on the RenderThread
    pub fn internal_task_immediate(&mut self, task: RenderTask) -> Option<RenderTaskReturn> {
        println!("InternalTaskImmediate! {}", self.command_id);
        // Create a new render command and send it to the separate thread
        let render_command = RenderCommand {
            message_id: self.command_id,
            input_task: task,
        };
        // Just get the command lol
        let output = command(render_command);
        // Increment
        self.command_id += 1;
        println!("InternalTaskImmediate {} success!", self.command_id - 1);
        Some(output)
    }
    // Get GPU object using it's specified name
    pub fn get_gpu_object(&self, name: &str) -> &GPUObject {
        // Convert the name to an ID first
        let id = self.gpu_objects_names[name];
        self.gpu_objects.get(&id).unwrap()
    }
    // Check if a GPU object exists
    pub fn gpu_object_valid(&self, name: &str) -> bool {
        self.gpu_objects_names.contains_key(name)
    }
}
// Renderers
impl Pipeline {
    // Add the renderer to the renderer (lol I need better name)
    pub fn add_renderer(pr: &mut PipelineRenderer, renderer: SharedData<(Renderer, veclib::Matrix4x4<f32>)>) -> GPUObject {
        let matrix = &renderer.object.1;
        let renderer = &renderer.object.0;
        let model = renderer.model.clone();
        let material = Self::create_material(renderer.material.clone());
        let renderer_gpuobject = RendererGPUObject(model, material, *matrix);
        let x = pr.add_renderer(renderer_gpuobject);
        println!("Index: {}", x);
        GPUObject::Renderer(x)
    }
    // Remove the renderer using it's renderer ID
    pub fn remove_renderer(pr: &mut PipelineRenderer, renderer_id: usize) {
        pr.remove_renderer(renderer_id);
    }
}

// The actual OpenGL tasks that are run on the render thread
impl Pipeline {
    pub fn create_compile_subshader(subshader: SharedData<SubShader>) -> GPUObject {
        let shader_type: u32;
        let subshader = subshader.object.as_ref();
        match subshader.subshader_type {
            SubShaderType::Vertex => shader_type = gl::VERTEX_SHADER,
            SubShaderType::Fragment => shader_type = gl::FRAGMENT_SHADER,
            SubShaderType::Compute => shader_type = gl::COMPUTE_SHADER,
        }
        unsafe {
            let program = gl::CreateShader(shader_type);
            // Compile the shader
            let cstring = CString::new(subshader.source.clone()).unwrap();
            let shader_source: *const i8 = cstring.as_ptr();
            gl::ShaderSource(program, 1, &shader_source, null());
            gl::CompileShader(program);
            // Check for any errors
            let mut info_log_length: i32 = 0;
            let info_log_length_ptr: *mut i32 = &mut info_log_length;
            gl::GetShaderiv(program, gl::INFO_LOG_LENGTH, info_log_length_ptr);
            // Print any errors that might've happened while compiling this subshader
            if info_log_length > 0 {
                let mut log: Vec<i8> = vec![0; info_log_length as usize + 1];
                gl::GetShaderInfoLog(program, info_log_length, std::ptr::null_mut::<i32>(), log.as_mut_ptr());
                println!("Error while compiling sub-shader {}!:", subshader.name);
                let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect();
                let string = String::from_utf8(printable_log).unwrap();

                println!("Error: \n\x1b[31m{}", string);
                println!("\x1b[0m");
                println!("{}", subshader.source);
                panic!();
            }

            println!("\x1b[32mSubshader {} compiled succsessfully!\x1b[0m", subshader.name);
            GPUObject::SubShader(SubShaderGPUObject(subshader.subshader_type, program))
        }
    }
    pub fn create_compile_shader(shader: SharedData<Shader>) -> GPUObject {
        let shader = shader.object.as_ref();
        unsafe {
            let program = gl::CreateProgram();

            // Attach the shaders
            for subshader_program in shader.linked_subshaders_programs.iter() {
                gl::AttachShader(program, subshader_program.1);
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
                println!("Error while finalizing shader {}!:", shader.name);
                let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect();
                let string = String::from_utf8(printable_log).unwrap();
                println!("Error: \n\x1b[31m{}", string);
                println!("\x1b[0m");
                panic!();
            }
            // Check if this a compute shader
            let compute_shader: bool = match shader.linked_subshaders_programs.get(0).unwrap().0 {
                SubShaderType::Compute => true,
                _ => false,
            };
            // Detach shaders
            for subshader_program in shader.linked_subshaders_programs.iter() {
                gl::DetachShader(program, subshader_program.1);
            }
            if !compute_shader {
                // Normal shader
                GPUObject::Shader(ShaderGPUObject(program))
            } else {
                // Compute shader
                GPUObject::ComputeShader(ComputeShaderGPUObject(program))
            }
        }
    }
    pub fn create_model(model: SharedData<Model>) -> GPUObject {
        let model = model.object.as_ref();
        let mut gpu_data = ModelGPUObject::default();
        unsafe {
            // Create the VAO
            gl::GenVertexArrays(1, &mut gpu_data.vertex_array_object);
            gl::BindVertexArray(gpu_data.vertex_array_object);

            // Create the EBO
            gl::GenBuffers(1, &mut gpu_data.element_buffer_object);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, gpu_data.element_buffer_object);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (model.triangles.len() * size_of::<u32>()) as isize,
                model.triangles.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Create the vertex buffer and populate it
            gl::GenBuffers(1, &mut gpu_data.vertex_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.vertex_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (model.vertices.len() * size_of::<f32>() * 3) as isize,
                model.vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Create the normals buffer
            gl::GenBuffers(1, &mut gpu_data.normal_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.normal_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (model.normals.len() * size_of::<f32>() * 3) as isize,
                model.normals.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            if !model.tangents.is_empty() {
                // And it's brother, the tangent buffer
                gl::GenBuffers(1, &mut gpu_data.tangent_buf);
                gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.tangent_buf);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (model.tangents.len() * size_of::<f32>() * 4) as isize,
                    model.tangents.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );
            }

            if !model.uvs.is_empty() {
                // The texture coordinates buffer
                gl::GenBuffers(1, &mut gpu_data.uv_buf);
                gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.uv_buf);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (model.uvs.len() * size_of::<f32>() * 2) as isize,
                    model.uvs.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );
            }

            if !model.colors.is_empty() {
                // Finally, the vertex colors buffer
                gl::GenBuffers(1, &mut gpu_data.color_buf);
                gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.color_buf);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (model.colors.len() * size_of::<f32>() * 3) as isize,
                    model.colors.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );
            }
            // Create the vertex attrib arrays
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.vertex_buf);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());

            // Normal attribute
            gl::EnableVertexAttribArray(1);
            gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.normal_buf);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, null());

            if !model.tangents.is_empty() {
                // Tangent attribute
                gl::EnableVertexAttribArray(2);
                gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.tangent_buf);
                gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, 0, null());
            }
            if !model.uvs.is_empty() {
                // UV attribute
                gl::EnableVertexAttribArray(3);
                gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.uv_buf);
                gl::VertexAttribPointer(3, 2, gl::FLOAT, gl::FALSE, 0, null());
            }
            if !model.colors.is_empty() {
                // Vertex color attribute
                gl::EnableVertexAttribArray(4);
                gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.color_buf);
            }
            gl::VertexAttribPointer(4, 3, gl::FLOAT, gl::FALSE, 0, null());
            // Unbind
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
        gpu_data.element_count = model.triangles.len();
        GPUObject::Model(gpu_data)
    }
    pub fn dispose_model(mut model: ModelGPUObject) {
        unsafe {
            // Delete the VBOs
            gl::DeleteBuffers(1, &mut model.vertex_buf);
            gl::DeleteBuffers(1, &mut model.normal_buf);
            gl::DeleteBuffers(1, &mut model.uv_buf);
            gl::DeleteBuffers(1, &mut model.tangent_buf);
            gl::DeleteBuffers(1, &mut model.color_buf);
            gl::DeleteBuffers(1, &mut model.element_buffer_object);

            // Delete the vertex array
            gl::DeleteVertexArrays(1, &mut model.vertex_array_object);
        }
    }
    pub fn generate_texture(texture: SharedData<Texture>) -> GPUObject {
        let mut pointer: *const c_void = null();
        let texture = texture.object.as_ref();
        if !texture.bytes.is_empty() {
            pointer = texture.bytes.as_ptr() as *const c_void;
        }
        let ifd = crate::get_ifd(texture._format, texture._type);

        // Get the tex_type based on the TextureDimensionType
        let tex_type = match texture.ttype {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::TextureArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };

        // It's a normal mutable texture
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
                TextureType::TextureArray(width, height, depth) => {
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
        let wrapping_mode: i32;
        match texture.wrap_mode {
            TextureWrapping::ClampToEdge => wrapping_mode = gl::CLAMP_TO_EDGE as i32,
            TextureWrapping::ClampToBorder => wrapping_mode = gl::CLAMP_TO_BORDER as i32,
            TextureWrapping::Repeat => wrapping_mode = gl::REPEAT as i32,
            TextureWrapping::MirroredRepeat => wrapping_mode = gl::MIRRORED_REPEAT as i32,
        }
        unsafe {
            // Now set the actual wrapping mode in the opengl texture
            gl::TexParameteri(tex_type, gl::TEXTURE_WRAP_S, wrapping_mode);
            gl::TexParameteri(tex_type, gl::TEXTURE_WRAP_T, wrapping_mode);
        }
        println!("RenderThread: Succsesfully generated texture {}", texture.name);
        GPUObject::Texture(TextureGPUObject(id, ifd, texture.ttype))
    }
    pub fn update_texture_size(texture: TextureGPUObject, ttype: TextureType) {
        // Check if the current dimension type matches up with the new one
        let ifd = texture.1;
        // This is a normal texture getting resized
        unsafe {
            match ttype {
                TextureType::Texture1D(width) => {
                    gl::BindTexture(gl::TEXTURE_1D, texture.0);
                    gl::TexImage1D(gl::TEXTURE_2D, 0, ifd.0, width as i32, 0, ifd.1, ifd.2, null());
                }
                TextureType::Texture2D(width, height) => {
                    gl::BindTexture(gl::TEXTURE_2D, texture.0);
                    gl::TexImage2D(gl::TEXTURE_2D, 0, ifd.0, width as i32, height as i32, 0, ifd.1, ifd.2, null());
                }
                TextureType::Texture3D(width, height, depth) => {
                    gl::BindTexture(gl::TEXTURE_3D, texture.0);
                    gl::TexImage3D(gl::TEXTURE_3D, 0, ifd.0, width as i32, height as i32, depth as i32, 0, ifd.1, ifd.2, null());
                }
                TextureType::TextureArray(_, _, _) => todo!(),
            }
        }
    }
    pub fn update_texture_data(texture: TextureGPUObject, bytes: Vec<u8>) {
        let mut pointer: *const c_void = null();
        if !bytes.is_empty() {
            pointer = bytes.as_ptr() as *const c_void;
        }

        let (internal_format, format, data_type) = texture.1;
        let tex_type = match texture.2 {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::TextureArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };

        unsafe {
            gl::BindTexture(tex_type, texture.0);
            match texture.2 {
                TextureType::Texture1D(width) => gl::TexImage1D(tex_type, 0, internal_format, width as i32, 0, format, data_type, pointer),
                // This is a 2D texture
                TextureType::Texture2D(width, height) => {
                    gl::TexImage2D(tex_type, 0, internal_format, width as i32, height as i32, 0, format, data_type, pointer);
                }
                // This is a 3D texture
                TextureType::Texture3D(width, height, depth) => {
                    gl::TexImage3D(tex_type, 0, internal_format, width as i32, height as i32, depth as i32, 0, format, data_type, pointer);
                }
                // This is a texture array
                TextureType::TextureArray(width, height, depth) => {
                    gl::TexStorage3D(tex_type, 10, internal_format as u32, width as i32, height as i32, depth as i32);
                    // We might want to do mipmap
                    for i in 0..depth {
                        let localized_bytes = bytes[(i as usize * height as usize * 4 * width as usize)..bytes.len()].as_ptr() as *const c_void;
                        gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0, i as i32, width as i32, height as i32, 1, format, data_type, localized_bytes);
                    }
                }
            }
        }
    }
    pub fn texture_fill_array(texture: TextureGPUObject, bytecount: usize) -> Vec<u8> {
        // Get the length of the vector
        let length: usize = match texture.2 {
            TextureType::Texture1D(x) => (x as usize),
            TextureType::Texture2D(x, y) => (x as usize * y as usize),
            TextureType::Texture3D(x, y, z) => (x as usize * y as usize * z as usize),
            TextureType::TextureArray(_, _, _) => todo!(),
        };
        // Get the byte size
        let byte_length = bytecount * length;

        // Create the vector
        let mut pixels: Vec<u8> = vec![0; byte_length];

        let tex_type = match texture.2 {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::TextureArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };

        // Actually read the pixels
        unsafe {
            // Bind the buffer before reading
            gl::BindTexture(tex_type, texture.0);
            let (_internal_format, format, data_type) = texture.1;
            gl::GetTexImage(tex_type, 0, format, data_type, pixels.as_mut_ptr() as *mut c_void);
        }
        pixels
    }
    pub fn run_compute(_compute: ComputeShaderGPUObject, _indices: (u16, u16, u16)) {
        todo!();
    }
    pub fn lock_compute(_compute: ComputeShaderGPUObject) {
        todo!();
    }
    pub fn create_material(material: Material) -> MaterialGPUObject {
        MaterialGPUObject(material.shader, material.uniforms, material.flags)
    }
}
