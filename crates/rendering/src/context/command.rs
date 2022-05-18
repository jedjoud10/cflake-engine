use std::marker::PhantomData;

use super::Context;

// A fence is a unique OpenGL object that lets the CPU know whenever we finish executing some tasks on the GPU
pub struct Fence(*const gl::types::__GLsync);

impl Fence {
    // Detect whether a fence has finished executing or not
    pub fn completed(&self, ctx: &Context) -> bool {
        unsafe {
            // This is called we do a bit of trolling
            match gl::ClientWaitSync(self.0, gl::SYNC_FLUSH_COMMANDS_BIT, 0) {
                gl::ALREADY_SIGNALED | gl::CONDITION_SATISFIED => true,
                gl::TIMEOUT_EXPIRED => false,
                gl::WAIT_FAILED | _ => panic!(""),
            }
        }
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteSync(self.0);
        }
    }
}

// This is a simple command stream that we can use to help facilitate flushing/executing some commands
// Everything within the command buffer will get flushed to the driver whenever we call flush()
pub struct CommandStream<O, F: FnOnce(&mut Context) -> O>(PhantomData<*const F>, O);

impl<O, F: FnOnce(&mut Context) -> O> CommandStream<O, F> {
    // Initialize a new command stream, and run everything within it
    // This will give back a command stream that we can flush/wait for
    pub fn new(ctx: &mut Context, func: F) -> Self {
        unsafe {
            // We do this just in case
            gl::Flush();
        }

        // Execute the code
        let out = func(ctx);
        Self(Default::default(), out)
    }

    /*
    // Flush the command stream and simply return a fence, telling us when all the tasks finish executing
    pub fn flush(self, ctx: &mut Context) -> (Fence, O) {
        // Create a new fence to detect when the tasks finish
        let fence = Fence(unsafe {
            gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0)
        });

        // And flush
        unsafe {
            gl::Flush();
        }

        // Return the fence
        fence
    }
    */

    // Force the execution of the command stream. This will stall the CPU since it will wait for the GPU to complete it's tasks
    pub fn wait(self, ctx: &mut Context) -> O {
        // Very simple
        unsafe {
            gl::Finish();
        }

        self.1
    }
}
