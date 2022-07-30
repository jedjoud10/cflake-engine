use crate::{canvas::Canvas, context::Context};

// This is a collection of post-processing effects that will 
// be rendered onto the screen after we render the basic scene
pub struct PostProcessing {
    canvas: Canvas,
    tonemapping_strength: f32,
    exposure: f32,
    vignette_strength: f32,
    vignette_size: f32,
}

impl PostProcessing {
    pub(crate) fn new(ctx: &mut Context, size: vek::Extent2<u16>) -> Self {
        Self { 
            canvas: Canvas::new(ctx, size, todo!()).unwrap(),
            tonemapping_strength: 1.0,
            exposure: 1.0, 
            vignette_strength: 1.0,
            vignette_size: 1.0
        }
    }

    pub(crate) fn resize(&mut self, size: vek::Extent2<u16>) {

    }
}
