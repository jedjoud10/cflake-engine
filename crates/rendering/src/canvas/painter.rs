use std::{intrinsics::transmute, mem::transmute_copy, ptr::null};

use super::{Canvas};
use crate::{
    context::Context,
    prelude::{Program, Shader, Uniforms, Populated}, others::Comparison, mesh::attributes::AttributeSet, buffer::ElementBuffer, object::ToGlName,
};

// Blend mode factor source
// This is a certified bruh moment classic
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum Factor {
    Zero = gl::ZERO,
    One = gl::ONE,
    SrcColor = gl::SRC_COLOR,
    DstColor = gl::DST_COLOR,
    SrcAlpha = gl::SRC_ALPHA,
    DstAlpha = gl::DST_ALPHA,
    OneMinusSrcColor = gl::ONE_MINUS_SRC_COLOR,
    OneMinusDstColor = gl::ONE_MINUS_DST_COLOR,
    OneMinusSrcAlpha = gl::ONE_MINUS_SRC_ALPHA,
    OneMinusDstAlpha = gl::ONE_MINUS_DST_ALPHA,
}

// Blending mode when utilising alpha blending moment
#[derive(Clone, Copy)]
pub struct BlendMode {
    pub(super) s_factor: Factor,
    pub(super) d_factor: Factor,
}

impl BlendMode {
    pub fn with(s_factor: Factor, d_factor: Factor) -> Self {
        Self { s_factor, d_factor }
    }
}


// How rasterized triangles should be culled
#[derive(Clone)]
pub enum FaceCullMode {
    // The boolean specifies if the culling should be Counter Clockwise
    Front(bool),
    Back(bool),

    // Don't cull anything
    None,
}

// Main rasterizer settings like sissor tests and depth tests
#[derive(Clone)]
pub struct RasterSettings {
    // Should we check for vertex depth when rasteizing?
    pub depth_test: Option<Comparison>,

    // A scissor test basically limits the area of effect when rasterizing. Pretty useful for UI
    pub scissor_test: Option<(vek::Vec2<i32>, vek::Extent2<i32>)>,

    // The current primitive that we will render with. Currently supported: Triangles and Points
    pub primitive: PrimitiveMode,

    // Should we render in SRGB or not?
    pub srgb: bool,

    // Transparancy blending mode
    pub blend: Option<BlendMode>,
}

// Depicts the exact primitives we will use to draw the mesh
#[derive(Clone)]
pub enum PrimitiveMode {
    Triangles { cull: FaceCullMode },
    Points { diameter: f32 },
}

// An object that can be rasterized and drawn onto the screen
pub trait ToRasterBuffers {
    // Get the VAO handle of the object
    fn vao(&self) -> &AttributeSet;

    // Get the EBO handle of the object
    fn ebo(&self) -> &ElementBuffer<u32>;
}

// A painter will help us render specific shaded / colored objects onto the screen
// Painters can be fetched from any canvas using the .paint() method to be able to draw onto that canvas
pub struct Painter<'canvas, 'context> {
    canvas: &'canvas mut Canvas,
    ctx: &'context mut Context,
    primitive: u32, 
}

impl<'canvas, 'context> Painter<'canvas, 'context> {
    // Create a new painter with the specified raster settings
    pub(crate) fn new(canvas: &'canvas mut Canvas, ctx: &'context mut Context, settings: RasterSettings) -> Self {
        // Get the OpenGL primitive type
        let primitive = match settings.primitive {
            PrimitiveMode::Triangles { .. } => gl::TRIANGLES,
            PrimitiveMode::Points { .. } => gl::POINTS,
        };

        // Set the global OpenGL face culling mode
        unsafe fn set_cull_mode(mode: &FaceCullMode) {
            // Check if we must cull the faces or not
            if let FaceCullMode::None = mode {
                gl::Disable(gl::CULL_FACE);
                return;
            } else {
                gl::Enable(gl::CULL_FACE)
            };

            // Get the face culling direction, either front or back, and winding order
            let (direction, ccw) = match mode {
                FaceCullMode::Front(ccw) => (gl::FRONT, ccw),
                FaceCullMode::Back(ccw) => (gl::BACK, ccw),
                _ => todo!(),
            };

            // Set the face culling direction
            gl::CullFace(direction);

            // And set winding order
            gl::FrontFace(if *ccw { gl::CCW } else { gl::CW });
        }

        // Set the global OpenGL point size
        unsafe fn set_point_size(diameter: &f32) {
            gl::PointSize(*diameter);
        }

        // Set the OpenGL primitive parameters
        match &settings.primitive {
            PrimitiveMode::Triangles { cull } => unsafe { set_cull_mode(cull) },
            PrimitiveMode::Points { diameter } => unsafe { set_point_size(diameter) },
        }

        // Handle depth testing and it's parameters
        unsafe {
            if let Some(func) = &settings.depth_test {
                gl::Enable(gl::DEPTH_TEST);
                gl::DepthFunc(transmute_copy::<Comparison, u32>(func));
            } else {
                gl::Disable(gl::DEPTH_TEST);
            }
        }

        // Handle scissor testing and it's parameters
        unsafe {
            if let Some((origin, size)) = &settings.scissor_test {
                gl::Enable(gl::SCISSOR_TEST);
                gl::Scissor(origin.x, canvas.size().h as i32 - origin.y, size.w, size.h);
            } else {
                gl::Disable(gl::SCISSOR_TEST);
            }
        }

        // Handle the SRGB framebuffer mode
        unsafe {
            if settings.srgb {
                gl::Enable(gl::FRAMEBUFFER_SRGB);
            } else {
                gl::Disable(gl::FRAMEBUFFER_SRGB);
            }
        }

        // Handle blending and it's parameters
        unsafe {
            if let Some(mode) = settings.blend {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(
                    transmute::<Factor, u32>(mode.s_factor),
                    transmute::<Factor, u32>(mode.d_factor),
                );
            } else {
                gl::Disable(gl::BLEND)
            }
        }

        // Create the painter object
        Self { canvas, ctx, primitive }
    }

    // Create a new painter pass that uses a shader and some uniforms
    pub fn pass<'shader>(&mut self, shader: &'shader mut Shader, populate: impl FnOnce(&mut Uniforms<'shader>)) -> Pass<'shader> {
        let populated = unsafe { crate::prelude::populate(self.ctx, shader.as_mut(), populate) };
        Pass { populated, primitive: self.primitive }
    }
}

// Painters use multiple "passes" to render different batches of object with different shaders / uniforms
// We can actually call the .draw() methods on passes, but not on painters directly
pub struct Pass<'shader> {
    populated: Populated<'shader>,
    primitive: u32,
}

impl<'shader> Pass<'shader> {
    // Rasterize a raw VAO and raw EBO using their OpenGL names, alongside the primitive count
    pub unsafe fn draw_from_raw_parts(&mut self, vao: u32, ebo: u32, count: u32) {
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::DrawElements(self.primitive, count as i32, gl::UNSIGNED_INT, null());
    }

    // Draw an object that implements the ToRasterBuffers. Get it's VAO, and EBO and render them.
    pub fn draw<T: ToRasterBuffers>(&mut self, obj: &T) {
        unsafe {
            self.draw_from_raw_parts(obj.vao().name(), obj.ebo().name(), obj.ebo().len() as u32)
        }
    }
}