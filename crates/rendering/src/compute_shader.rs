use crate::ArrayData;

// Compute shader containing the compute shader code
#[derive(Default)]
pub struct ComputeShader {
    pub running: bool,
    // Do we have any array data?
    // TODO: Make this support multiple array data
    pub array_data: Option<ArrayData>,
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
        return Some(());
    }
    // Force the compute shader to finish running if it is still running
    pub fn get_compute_state(&mut self) -> Option<()> {
        unsafe {
            if self.running {
                // Force the compute shader to complete
                //gl::Finish();
                gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
                //gl::MemoryBarrier(gl::ALL_BARRIER_BITS);
                self.running = false;
            } else {
                return None;
            }
        }
        return Some(());
    }
    // Create some array data with a specific max size and a specific binding
    pub fn create_array_data<T>(&mut self, max_size: usize) {
        // Create some array data
        let mut ad = ArrayData::default();
        ad.create_array::<T>(max_size);
        self.array_data = Some(ad);
    }
    // Read back some array data that was ran inside the compute shader
    pub fn read_array_data<T: Sized + Clone>(&mut self) -> Option<Vec<T>> {
        match self.array_data.as_mut() {
            Some(x) => {
                let x = x.read::<T>();
                return Some(x);
            }
            None => todo!(),
        }
        // Were we sucsessful?
        return None;
    }
}
