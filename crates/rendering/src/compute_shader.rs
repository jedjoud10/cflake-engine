// Compute shader containing the compute shader code
#[derive(Default)]
pub struct ComputeShader {
    pub running: bool,
}

// Compute shader code
impl ComputeShader {
    // Run the compute shader if this shader is a compute shader
    // TODO: This runs very oddly on integrated GPU, it seems like the depth is always 1 for that case
    pub fn run_compute(&mut self, num_groups: (u32, u32, u32)) -> Option<()> {
        if self.running {
            return Some(());
        }
        unsafe {
            // Do some num_groups checks
            gl::DispatchCompute(num_groups.0, num_groups.1, num_groups.2);
            self.running = true;
        }
        errors::ErrorCatcher::catch_opengl_errors()?;
        return Some(());
    }
    // Force the compute shader to finish running if it is still running
    pub fn get_compute_state(&mut self) -> Option<()> {
        unsafe {
            if self.running {
                // Force the compute shader to complete
                gl::Finish();
                gl::MemoryBarrier(gl::ALL_BARRIER_BITS);
                errors::ErrorCatcher::catch_opengl_errors()?;
                self.running = false;
            } else {
            }
        }
        errors::ErrorCatcher::catch_opengl_errors()?;
        return Some(());
    }
}
