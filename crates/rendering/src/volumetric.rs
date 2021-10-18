use crate::{Texture2D, Texture3D};

// Some volumetric shit
pub struct Volumetric {
    // The main SDF texture used for the volumetric sampling
    pub std_tex: Texture3D,
    // The output, screen texture that will be rendered (PS: This texture might be downscaled from the original screen size)
    pub result_tex: Texture2D,
    // The compute shader ID
    pub compute_id: usize,
}

impl Volumetric {
    // Create the SDF texture from a simple texture, loaded into a compute shader
    // Create the SDF texture from a compute shader complitely
    // Run the compute shader and calculate the result texture
    pub fn calculate_volumetric(&mut self, projection_matrix: veclib::Matrix4x4<f32>, view_matrix: veclib::Matrix4x4<f32>, camera_position: veclib::Vector3<f32>) {
        // Run the compute shader
    }
}