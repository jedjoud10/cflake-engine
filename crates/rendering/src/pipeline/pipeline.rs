use glfw::Context;

use crate::{basics::*, RenderCommand, RenderTask, RenderTaskStatus};
use std::sync::{Arc, Mutex, mpsc::{Receiver, Sender}};

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
    // The render thread that is continuously being ran
    fn frame(render_to_main: Sender<RenderTaskStatus>, main_to_render: Receiver<RenderCommand>) {}
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
