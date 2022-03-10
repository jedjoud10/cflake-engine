// Some compute shader settings that we can use whenever we want to execute a compute shader
pub struct ComputeShaderExecutionSettings {
    // We must know the axii groups
    pub axii: veclib::Vector3<u16>,
}

impl ComputeShaderExecutionSettings {
    // Create some new settings using the axii counts
    pub fn new(axii: veclib::Vector3<u16>) -> Self {
        Self { axii }
    }
}
