use glfw::Context;

use crate::{RenderCommand, RenderTask, RenderTaskStatus, SharedGPUObject, basics::*};
use std::{ffi::CString, ptr::null, sync::{Arc, Mutex, mpsc::{Receiver, Sender}}};

// Render pipeline. Contains everything related to rendering. This is also ran on a separate thread
#[derive(Default)]
pub struct RenderPipeline {
    // Command ID
    pub command_id: u128,
    // The tasks that are asynchronous and are pending their return values
    pub pending_wait_list: Vec<(RenderCommand, Box<dyn FnMut(GPUObject)>)>,
    // TX (RenderThread) and RX (MainThread)
    pub render_to_main: Option<(Sender<RenderTaskStatus>, Receiver<RenderTaskStatus>)>,
    // TX (MainThread) and RX (RenderThread)
    pub main_to_render: Option<(Sender<RenderCommand>, Receiver<RenderCommand>)>,
}
impl RenderPipeline {
    // Run a command
    fn command(command: RenderCommand, render_to_main: Sender<RenderTaskStatus>) {
        let object = match command.input_task {
            RenderTask::DisposeRenderer(_) => todo!(),
            RenderTask::UpdateRendererTransform() => todo!(),
            RenderTask::CreateSubShader(x) => Self::create_compile_subshader(x),
            RenderTask::CreateShader(_) => todo!(),
            RenderTask::GenerateTexture(_) => todo!(),
            RenderTask::RefreshModel(_) => todo!(),
            RenderTask::RunCompute() => todo!(),
            RenderTask::DestroyRenderPipeline() => todo!(),
        };
        // Send back a possible new GPU object
        let object = match object {
            GPUObject::None => None,
            _ => Some(object),
        };
        // Send back the message to the main thread
        let status = RenderTaskStatus::Succsessful(object);
        render_to_main.send(status).unwrap();
    }
    // The render thread that is continuously being ran
    fn frame(render_to_main: Sender<RenderTaskStatus>, main_to_render: Receiver<RenderCommand>) {
        // We must loop through every command that we receive from the main thread
        loop {
            match main_to_render.try_recv() {
                Ok(x) => {
                    // Valid command
                    Self::command(x, render_to_main);
                },
                Err(x) => match x {
                    std::sync::mpsc::TryRecvError::Empty => /* Quit from the loop, we don't have any commands stacked up for this frame */ break,
                    std::sync::mpsc::TryRecvError::Disconnected => /* The channel got disconnected */ {},
                },
            } 
        }
    }
    // Create the new render thread
    pub fn init_pipeline(&mut self, glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
        // Create the two channels
        let (tx, rx): (Sender<RenderTaskStatus>, Receiver<RenderTaskStatus>) = std::sync::mpsc::channel(); // Render to main
        let (tx2, rx2): (Sender<RenderCommand>, Receiver<RenderCommand>) = std::sync::mpsc::channel(); // Main to render
        let render_thread = unsafe {
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
                gl::load_with(|s| window.get_proc_address(s) as *const _);
                glfw::ffi::glfwMakeContextCurrent(window.window_ptr() as *mut glfw::ffi::GLFWwindow);
                // We must render every frame
                loop {
                    Self::frame(tx, rx2)
                }
            })
        };
        // Vars
        self.render_to_main = Some((tx, rx));
    }
    // Complete a task immediatly
    pub fn task_immediate(&mut self, task: RenderTask) -> GPUObject {
        // Create a new render command and send it to the separate thread
        let render_command = RenderCommand {
            message_id: self.command_id,
            input_task: task,
        };
        // Increment
        self.command_id += 1;
        // Send the command to the render thread
        let tx = self.main_to_render.unwrap().0;
        let rx = self.render_to_main.unwrap().1;

        // Send the command
        tx.send(render_command);
        // Wait for the result
        let recv = rx.recv().unwrap();
        let output = GPUObject::None;
        return output;
    }
    // Complete a task, but the result is not needed immediatly, and call the call back when the task finishes
    pub fn task<F>(&mut self, task: RenderTask, mut callback: F)
    where
        F: FnMut(GPUObject) + 'static,
    {
        let boxed_fn_mut: Box<dyn FnMut(GPUObject)> = Box::new(callback);
        // Create a new render command and send it to the separate thread
        let render_command = RenderCommand {
            message_id: self.command_id,
            input_task: task,
        };
        // Increment
        self.command_id += 1;
        // Send the command to the render thread
        let tx = self.main_to_render.unwrap().0;
        let rx = self.render_to_main.unwrap().1;

        // Send the command
        tx.send(render_command);
        // This time, we must add this to the wait list
        self.pending_wait_list.push((render_command, boxed_fn_mut));
    }
}
// The actual OpenGL tasks that are run on the render thread
impl RenderPipeline {
    fn create_compile_subshader(subshader: SharedGPUObject<SubShader>) -> GPUObject {
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
            GPUObject::SubShader(subshader.subshader_type, program)
        }
    }
    fn create_compile_shader(shader: SharedGPUObject<Shader>) -> GPUObject {
        let shader = shader.object.as_ref();
        unsafe {
            let program = gl::CreateProgram();
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

            // Detach shaders
            for subshader_program in shader.linked_subshaders_programs.iter() {
                match subshader_program {
                    GPUObject::SubShader(x, i) => gl::DetachShader(program, *i),
                    _ => {}
                }
            }
            GPUObject::Shader(program)
        }
    }
}