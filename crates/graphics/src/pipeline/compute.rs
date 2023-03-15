use crate::{ComputeShader, Graphics, PipelineInitializationError};

// Wrapper around a WGPU compute pipeline just to help me instantiate them
pub struct ComputePipeline {
    pipeline: wgpu::ComputePipeline,

    // Immutable data set at build time

    // Keep the compute shader alive
    shader: ComputeShader,

    // Keep the graphics API alive
    graphics: Graphics,
}

impl ComputePipeline {
    // Create a new pipeline with the specified configs
    pub fn new(
        graphics: &Graphics,
        shader: &ComputeShader,
    ) -> Result<Self, PipelineInitializationError> {
        let pipeline = graphics.device().create_compute_pipeline(
            &wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(&shader.layout),
                module: shader.compute().module(),
                entry_point: "main",
            },
        );

        Ok(Self {
            pipeline,
            shader: shader.clone(),
            graphics: graphics.clone(),
        })
    }

    // Get the underlying raw WGPU compute pipeline
    pub fn pipeline(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }

    // Get the internally used compute shader for this compute pipeline
    pub fn shader(&self) -> &ComputeShader {
        &self.shader
    }
}
