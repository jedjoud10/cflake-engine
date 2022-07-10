use std::{intrinsics::transmute, mem::transmute_copy, ptr::null};

use super::{Canvas, RasterError};
use crate::{
    buffer::ElementBuffer, context::Context, object::ToGlName,
    others::Comparison, prelude::Uniforms, mesh::Mesh,
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

// Tells us if we how we should blend between transparent objects
#[derive(Clone, Copy)]
pub struct BlendMode {
    pub src: Factor,
    pub dest: Factor,
}

// How rasterized triangles should be culled
#[derive(Clone, Copy)]
pub enum FaceCullMode {
    Front(bool),
    Back(bool),
}

// Main rasterizer self like sissor tests and depth tests
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
    Triangles { cull: Option<FaceCullMode> },
    Points { diameter: f32 },
}

// A rasterizer will help us render specific shaded / colored objects onto the screen
// Painters can be fetched from any mutable reference to a canvas
pub struct Rasterizer<'canvas, 'context> {
    canvas: &'canvas mut Canvas,
    ctx: &'context mut Context,
    primitive: u32,
}

impl<'canvas, 'context> Rasterizer<'canvas, 'context> {
    // Create a new rasterizer with the specified raster self
    pub(crate) fn new(
        canvas: &'canvas mut Canvas,
        ctx: &'context mut Context,
        settings: RasterSettings,
    ) -> Self {
        // Get the OpenGL primitive type
        let primitive = match settings.primitive {
            PrimitiveMode::Triangles { .. } => gl::TRIANGLES,
            PrimitiveMode::Points { .. } => gl::POINTS,
        };

        // Set the OpenGL primitive parameters (along with face culling)
        match &settings.primitive {
            PrimitiveMode::Triangles { cull } => unsafe {
                if let Some(cull) = cull {
                    gl::Enable(gl::CULL_FACE);

                    // Get the face culling direction, either front or back, and winding order
                    let (direction, ccw) = match cull {
                        FaceCullMode::Front(ccw) => (gl::FRONT, ccw),
                        FaceCullMode::Back(ccw) => (gl::BACK, ccw),
                    };

                    // Set the face culling direction
                    gl::CullFace(direction);

                    // And set winding order
                    gl::FrontFace(if *ccw { gl::CCW } else { gl::CW });
                } else {
                    gl::Disable(gl::CULL_FACE);
                };
            },
            PrimitiveMode::Points { diameter } => unsafe {
                gl::PointSize(*diameter);
            },
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
                    transmute::<Factor, u32>(mode.src),
                    transmute::<Factor, u32>(mode.dest),
                );
            } else {
                gl::Disable(gl::BLEND)
            }
        }

        Self {
            canvas,
            ctx,
            primitive,
        }
    }

    // Get an immutable reference to the underlying canvas
    pub fn canvas(&self) -> &Canvas {
        self.canvas
    }

    // Get an immutable reference to the underlying context
    pub fn context(&self) -> &Context {
        self.ctx
    }

    // Draw a raw vertex array object, and an element array object
    pub unsafe fn draw_ebo_vao(
        &mut self,
        vao: u32,
        ebo: u32,
        count: usize,
        uniforms: &mut Uniforms,
    ) -> Result<(), RasterError> {
        uniforms.validate().map_err(RasterError::Uniforms)?;

        if count > 0 {
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::DrawElements(self.primitive, count as i32, gl::UNSIGNED_INT, null());
        }

        Ok(())
    }

    // Draw a raw vertex array object by itself
    pub unsafe fn draw_vao(
        &mut self,
        vao: u32,
        count: usize,
        uniforms: &mut Uniforms
    ) -> Result<(), RasterError> {
        uniforms.validate().map_err(RasterError::Uniforms)?;

        if count > 0 {
            gl::BindVertexArray(vao);
            gl::DrawArrays(self.primitive, 0, count as i32);
        }

        Ok(())
    }

    // Draw a 3D engine mesh directly 
    pub fn draw(&mut self, mesh: &Mesh, uniforms: &mut Uniforms) -> Result<(), RasterError> {
        unsafe {
            self.draw_ebo_vao(mesh.vao, mesh.indices().name(), mesh.indices().len(), uniforms)
        }
    } 
}
