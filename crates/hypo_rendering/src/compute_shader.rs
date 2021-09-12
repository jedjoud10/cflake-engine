// Compute shader containing the compute shader code
#[derive(Default)]
pub struct ComputeShader {}

// Compute shader code
impl ComputeShader {
    // Run the compute shader if this shader is a compute shader
    pub fn run_compute(&self, num_groups: (u32, u32, u32)) {
        unsafe {
            // Do some num_groups checks
            let mut max: i32 = 0;
            gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_COUNT, 0, &mut max);
            if num_groups.0 * num_groups.1 * num_groups.2 > max as u32 {
                // We have exceeded the max, this is not good
                panic!("Num groups dispatched for compute shader are invalid!");
            }
            gl::DispatchCompute(num_groups.0, num_groups.1, num_groups.2);
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
        }
    }
}
