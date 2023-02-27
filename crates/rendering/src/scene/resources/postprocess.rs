use assets::Assets;
use graphics::{SwapchainFormat, RenderPass, LoadOp, StoreOp, Operation, Graphics, Shader, VertexModule, FragmentModule, Compiler, GraphicsPipeline, VertexConfig, PrimitiveConfig};

// Container for post-processing parameters
pub struct PostProcess {
    // Lighting parameters
    pub exposure: f32,
    pub gamma: f32,

    // Vignette parameters
    pub vignette_strength: f32,
    pub vignette_size: f32,
} 

impl Default for PostProcess {
    fn default() -> Self {
        Self {
            exposure: 1.2,
            gamma: 2.2,
            vignette_strength: 1.0,
            vignette_size: 1.0,
        }
    }
}