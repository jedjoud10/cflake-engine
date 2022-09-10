use std::time::Duration;

use crate::context::Context;

// This Gpu timer will tell us how much time it took to execute a specific function on the GPU
// This timer will contain an OpenGL timer query and a OpenGL fence
pub struct CommandTimer {
    query: u32,
    fence: gl::types::GLsync,
}

impl CommandTimer {
    // Create a new command timer that we can use multiple times
    pub fn 

    // Create a new fence and run the closure within it
    pub fn new(ctx: &Context, closure: impl FnOnce()) -> Self {
        unsafe {
            let mut query = 0u32;
            gl::Flush();
            gl::CreateQueries(gl::TIME_ELAPSED, 1, &mut query);
            gl::BeginQuery(gl::TIME_ELAPSED, query);
            closure();
            gl::EndQuery(gl::TIME_ELAPSED);

            let fence = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
        
            Self {
                query,
                fence,
            }
        }
    }    
    
    // Check if the OpenGL fence has been signaled
    pub fn signaled(&self) -> bool {
        let state = unsafe { gl::ClientWaitSync(self.fence, 0, 0) };
        match state {
            gl::ALREADY_SIGNALED | gl::CONDITION_SATISFIED => true,
            gl::WAIT_FAILED | gl::TIMEOUT_EXPIRED => false,
            _ => false,
        }
    }

    // Check if all the commands have completed, and return their execution time (on the GPU!)
    pub fn elapsed(&self) -> Option<Duration> {
        self.signaled().then(|| unsafe {
            let mut result = 0u64;
            gl::GetQueryObjectui64v(self.query, gl::QUERY_RESULT_NO_WAIT, &mut result);
            Duration::from_nanos(result)
        })
    }
}