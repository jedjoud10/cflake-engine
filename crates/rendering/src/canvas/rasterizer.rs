use crate::{
    buffer::ElementBuffer,
    canvas::blend::{BlendMode, Factor},
    canvas::Canvas,
    context::Context,
    mesh::attributes::AttributeSet,
    object::ToGlName,
    others::Comparison,
    shader::{Shader, Uniforms},
};
use std::{
    mem::{transmute, transmute_copy},
    ptr::null,
};



// A rasterizer is a collection of settings that "Pass"es will use to render something pwetty
// Rasterizer can create multiple "pass"es that use a specific shader and shit
pub struct Painter {
    settings: RasterSettings,
    primitive: u32,
}

impl Painter {
    // Create a new rasterizer with the given raw fields
    // This has to be a function since we run the "apply settings" shit here
    pub(super) fn new(
        canvas: &Canvas,
        context: &Context,
        settings: RasterSettings,
    ) -> Self {
        

        // Create the rasterizer object
        Self {
            settings,
            primitive,
        }
    }

    // Create a new rasterizer pass that uses a specific shader and sets it's uniforms
    // I separated the actual rendering part from this since we might want to render a bunch of objects, but with different shaders
    pub fn pass(&self, shader: &mut Shader, populate: impl FnOnce(&mut Uniforms)) -> RasterizerPass {

    } 

    
}


// A rasterizer pass is the raw interface that will actually draw buffers onto the screen
// Rasterizer passes have a notion of what is a shader, so they can use that knowledge to set the specific shader uniforms and such